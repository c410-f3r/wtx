use crate::{
  http::{RequestRef, ResponseMut},
  http2::{
    misc::{send_go_away, send_reset, write_init_headers},
    ErrorCode, GoAwayFrame, HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer,
    Http2Data, ReadFrameRslt, ReqResBuffer, ResetStreamFrame, StreamState, U31,
  },
  misc::{Lease, LeaseMut, Lock, RefCounter, Stream, Usize},
};
use core::marker::PhantomData;

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<HB, HD, S> {
  hd: HD,
  phantom: PhantomData<(HB, S)>,
  stream_id: U31,
  stream_state: StreamState,
}

impl<HB, HD, S> ClientStream<HB, HD, S> {
  pub(crate) fn idle(hd: HD, stream_id: U31) -> Self {
    Self { phantom: PhantomData, hd, stream_id, stream_state: StreamState::Idle }
  }
}

impl<HB, HD, S> ClientStream<HB, HD, S>
where
  HB: LeaseMut<Http2Buffer<true>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, S, true>>,
  S: Stream,
{
  /// Receive Response
  ///
  /// Composes all header frames, data frames and trailer frames of an received stream.
  ///
  /// Should be called onl once after sending a request.
  #[inline]
  pub async fn recv_res<'rrb>(
    &mut self,
    rrb: &'rrb mut ReqResBuffer,
  ) -> crate::Result<ReadFrameRslt<ResponseMut<'rrb, ReqResBuffer>>> {
    rrb.clear();
    let status_code = rfr_until_resource_with_guard!(self.hd, |guard| {
      guard
        .read_frames_stream(&mut *rrb, self.stream_id, &mut self.stream_state, |hf| {
          hf.hsresh().status_code.ok_or(crate::Error::MissingResponseStatusCode)
        })
        .await?
    });
    self.stream_state = StreamState::Closed;
    Ok(ReadFrameRslt::Resource(ResponseMut::http2(rrb, status_code)))
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  pub async fn send_go_away(&mut self, error_code: ErrorCode) -> crate::Result<()> {
    send_go_away(
      GoAwayFrame::new(error_code, self.stream_id),
      self.hd.lock().await.is_conn_open_and_stream_mut(),
    )
    .await
  }

  /// Send Request
  ///
  /// Composes all header frames, data frames and trailer frames that will be sent in a stream.
  ///
  /// Shouldn't be called more than one time.
  #[inline]
  pub async fn send_req<D>(&mut self, req: RequestRef<'_, '_, '_, D>) -> crate::Result<()>
  where
    D: Lease<[u8]>,
  {
    let mut guard = self.hd.lock().await;
    if req.data.lease().len() > *Usize::from(guard.send_params_mut().max_expanded_headers_len) {
      return Err(crate::Error::VeryLargeHeadersLen);
    }
    let (hb, is_conn_open, send_params, stream) = guard.parts_mut();
    write_init_headers::<_, true>(
      req.data.lease(),
      req.headers,
      &mut hb.hpack_enc,
      &mut hb.hpack_enc_buffer,
      (
        HpackStaticRequestHeaders {
          authority: req.uri.authority().as_bytes(),
          method: Some(req.method),
          path: req.uri.href().as_bytes(),
          protocol: None,
          scheme: req.uri.schema().as_bytes(),
        },
        HpackStaticResponseHeaders::EMPTY,
      ),
      *is_conn_open,
      send_params,
      stream,
      self.stream_id,
    )
    .await?;
    self.stream_state = StreamState::HalfClosedLocal;
    Ok(())
  }

  /// Groups [Self::send_req] and [Self::recv_res] in a single method.
  #[inline]
  pub async fn send_req_recv_res<'rrb, D>(
    &mut self,
    req: RequestRef<'_, '_, '_, D>,
    rrb: &'rrb mut ReqResBuffer,
  ) -> crate::Result<ReadFrameRslt<ResponseMut<'rrb, ReqResBuffer>>>
  where
    D: Lease<[u8]>,
  {
    self.send_req(req).await?;
    self.recv_res(rrb).await
  }

  /// Sends a RST_STREAM frame to the peer, which cancels this stream.
  pub async fn send_reset(&mut self, error_code: ErrorCode) -> crate::Result<()> {
    send_reset(
      ResetStreamFrame::new(error_code, self.stream_id)?,
      &mut self.stream_state,
      self.hd.lock().await.is_conn_open_and_stream_mut(),
    )
    .await
  }
}
