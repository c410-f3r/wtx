use crate::{
  http::{Method, RequestMut, Response, ResponseData},
  http2::{
    http2_data::ReadFramesInit,
    misc::{send_go_away, send_reset, write_to_stream},
    DataFrame, ErrorCode, GoAwayFrame, HeadersFrame, HpackStaticRequestHeaders,
    HpackStaticResponseHeaders, Http2Buffer, Http2Data, ReadFrameRslt, ReqResBuffer,
    ResetStreamFrame, StreamState, U31,
  },
  misc::{ByteVector, Lease, LeaseMut, Lock, RefCounter, Stream, Uri},
};
use core::marker::PhantomData;
use tokio::sync::MutexGuard;

/// Created when a server receives an initial stream. Used mainly to poll remaining data but can
/// also be used to send additional streams.
#[derive(Debug)]
pub struct ServerStream<HB, HD, S> {
  hd: HD,
  hpack_size: usize,
  is_eos: bool,
  method: Method,
  phantom: PhantomData<(HB, S)>,
  stream_id: U31,
  stream_state: StreamState,
}

impl<HB, HD, S> ServerStream<HB, HD, S> {
  #[inline]
  pub(crate) fn new(hd: HD, rfi: ReadFramesInit<Method>, stream_state: StreamState) -> Self {
    Self {
      hd,
      hpack_size: rfi.hpack_size,
      is_eos: rfi.is_eos,
      method: rfi.headers_rslt,
      phantom: PhantomData,
      stream_id: rfi.stream_id,
      stream_state,
    }
  }
}

impl<HB, HD, S> ServerStream<HB, HD, S>
where
  HB: LeaseMut<Http2Buffer<false>>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, false>>,
      Resource = Http2Data<HB, S, false>,
    > + 'guard,
  S: Stream,
{
  /// High-level method that reads all remaining data to build a request.
  #[inline]
  pub async fn recv_req<'rrb>(
    &mut self,
    rrb: &'rrb mut ReqResBuffer,
  ) -> crate::Result<ReadFrameRslt<RequestMut<'rrb, 'rrb, 'rrb, ByteVector>>> {
    rrb.clear();
    rfr_until_resource_with_guard!(self.hd, |guard| {
      guard
        .read_frames_others(
          &mut self.hpack_size,
          self.is_eos,
          rrb,
          self.stream_id,
          &mut self.stream_state,
        )
        .await?
    });
    self.stream_state = StreamState::HalfClosedRemote;
    Ok(ReadFrameRslt::Resource(RequestMut::http2(
      &mut rrb.data,
      &mut rrb.headers,
      self.method,
      Uri::new(&*rrb.uri),
    )))
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  pub async fn send_go_away(self, error_code: ErrorCode) -> crate::Result<()> {
    send_go_away(
      GoAwayFrame::new(error_code, self.stream_id),
      self.hd.lock().await.is_conn_open_and_stream_mut(),
    )
    .await
  }

  /// Auxiliary high-level method that sends a response.
  #[inline]
  pub async fn send_res<D>(&mut self, res: Response<D>) -> crate::Result<()>
  where
    D: ResponseData,
    D::Body: Lease<[u8]>,
  {
    if !self.stream_state.can_server_send() {
      return Ok(());
    }
    let mut guard = self.hd.lock().await;
    let (hb, is_conn_open, _, stream) = guard.parts_mut();
    let mut hf = HeadersFrame::new(
      res.data.headers(),
      (
        HpackStaticRequestHeaders::EMPTY,
        HpackStaticResponseHeaders { status_code: Some(res.status_code) },
      ),
      self.stream_id,
    );
    let body = res.data.body().lease();
    if body.is_empty() {
      hf.set_eos();
      hf.write::<false>(&mut hb.hpack_enc, &mut hb.hpack_enc_buffer)?;
      write_to_stream([&*hb.hpack_enc_buffer], *is_conn_open, stream).await?;
    } else {
      hf.write::<false>(&mut hb.hpack_enc, &mut hb.hpack_enc_buffer)?;
      let data_len = u32::try_from(body.len())?;
      let df = DataFrame::eos(body, data_len, self.stream_id);
      write_to_stream(
        [&*hb.hpack_enc_buffer, df.bytes().as_slice(), df.data()],
        *is_conn_open,
        stream,
      )
      .await?;
    }
    self.stream_state = StreamState::Closed;
    Ok(())
  }

  /// Sends a stream reset to the peer, which cancels this stream.
  pub async fn send_reset(&mut self, error_code: ErrorCode) -> crate::Result<()> {
    send_reset(
      ResetStreamFrame::new(error_code, self.stream_id)?,
      &mut self.stream_state,
      self.hd.lock().await.is_conn_open_and_stream_mut(),
    )
    .await
  }
}
