use crate::{
  http::{RequestRef, Response, ResponseMut, StatusCode},
  http2::{
    DataFrame, FrameInit, HeadersFrame, HpackDecoder, HpackStaticRequestHeaders,
    HpackStaticResponseHeaders, Http2Data, ReqResBuffer, StreamId, UriBuffer,
  },
  misc::{Lock, RefCounter, SingleTypeStorage, Usize},
};
use core::marker::PhantomData;

pub struct ClientStream<S, SDC, const IS_CLIENT: bool> {
  phantom: PhantomData<S>,
  sdrc: SDC,
  stream_id: StreamId,
}

impl<S, SDC, SDL, const IS_CLIENT: bool> ClientStream<S, SDC, IS_CLIENT>
where
  S: crate::misc::Stream,
  SDC: RefCounter<SDL> + SingleTypeStorage<Item = SDL>,
  SDL: Lock<Http2Data<S, IS_CLIENT>>,
{
  pub(crate) fn new(sdrc: SDC, stream_id: StreamId) -> Self {
    Self { phantom: PhantomData, sdrc, stream_id }
  }
}

impl<S, SDC, SDL> ClientStream<S, SDC, true>
where
  S: crate::misc::Stream,
  SDC: RefCounter<SDL> + SingleTypeStorage<Item = SDL>,
  SDL: Lock<Http2Data<S, true>>,
{
  #[inline]
  pub async fn send_req<'rrb, D>(
    &self,
    req: RequestRef<'_, '_, '_, D>,
    rrb: &'rrb mut ReqResBuffer,
  ) -> crate::Result<ResponseMut<'rrb, ReqResBuffer>> {
    rrb.clear();
    let mut lock = self.sdrc.lock().await;
    let max_header_list_size = lock.max_header_list_size();
    let (hb, stream) = lock.parts_mut();

    HeadersFrame::new(
      req.headers,
      HpackStaticRequestHeaders::new(
        req.uri.authority().as_bytes(),
        Some(req.method),
        req.uri.href().as_bytes(),
        None,
        req.uri.schema().as_bytes(),
      ),
      HpackStaticResponseHeaders::EMPTY,
      self.stream_id,
    )
    .write::<true>(&mut hb.hpack_enc, &mut hb.wb)?;
    stream
      .write_vectored(&[
        &*hb.wb,
        DataFrame::new(&rrb.data, u32::try_from(rrb.data.len())?, self.stream_id)
          .bytes()
          .as_slice(),
        &rrb.data,
      ])
      .await?;

    let status_code = lock
      .read_frame((max_header_list_size, &mut *rrb), self.stream_id, {
        async fn fun(
          ((max_header_list_size, rrb), hpack_dec, fi, data, uri_buffer): (
            (u32, &mut ReqResBuffer),
            &mut HpackDecoder,
            FrameInit,
            &[u8],
            &mut UriBuffer,
          ),
        ) -> crate::Result<StatusCode> {
          let headers_frame = HeadersFrame::read::<true, false>(
            data,
            fi,
            &mut rrb.headers,
            hpack_dec,
            *Usize::from(max_header_list_size),
            &mut rrb.uri,
            uri_buffer,
          )?;
          Ok(headers_frame.hsresh().status_code.ok_or(crate::Error::MissingResponseStatusCode)?)
        }
        fun
      })
      .await?;

    lock
      .read_frame(&mut *rrb, self.stream_id, {
        async fn fun(
          (rrb, _, fi, data, _): (
            &mut ReqResBuffer,
            &mut HpackDecoder,
            FrameInit,
            &[u8],
            &mut UriBuffer,
          ),
        ) -> crate::Result<()> {
          rrb.data.extend_from_slice(DataFrame::read(data, fi)?.data);
          Ok(())
        }
        fun
      })
      .await?;

    Ok(Response::http2(rrb, status_code))
  }
}
