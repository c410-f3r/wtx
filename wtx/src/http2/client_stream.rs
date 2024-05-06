use crate::{
  http::{RequestRef, ResponseMut},
  http2::{
    misc::{send_go_away, send_reset},
    write_stream::write_stream,
    ErrorCode, GoAwayFrame, HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer,
    Http2Data, ReadFrameRslt, ReqResBuffer, ResetStreamFrame, StreamState, U31,
  },
  misc::{AsyncBounds, Lease, LeaseMut, Lock, RefCounter, Stream, _Span},
};
use core::marker::PhantomData;

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<HB, HD, S> {
  hd: HD,
  phantom: PhantomData<(HB, S)>,
  span: _Span,
  stream_id: U31,
  stream_state: StreamState,
}

impl<HB, HD, S> ClientStream<HB, HD, S> {
  pub(crate) fn idle(hd: HD, span: _Span, stream_id: U31) -> Self {
    Self { phantom: PhantomData, hd, span, stream_id, stream_state: StreamState::Idle }
  }
}

impl<HB, HD, S> ClientStream<HB, HD, S>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, S, true>>,
  S: AsyncBounds + Stream,
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
    let _e = self.span._enter();
    _trace!("Receiving response");
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
    D: Lease<[u8]> + ?Sized,
  {
    let _e = self.span._enter();
    _trace!("Sending request");
    let mut guard = self.hd.lock().await;
    let (hb, is_conn_open, send_params, stream) = guard.parts_mut();
    write_stream::<_, true>(
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
      &mut self.stream_state,
    )
    .await?;
    self.stream_state = StreamState::HalfClosedLocal;
    Ok(())
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
