use crate::{
  http::{Headers, Method, ReqResData, Request, Response},
  http2::{
    http2_data::ReadFramesInit,
    misc::{send_go_away, send_reset, verify_before_send},
    window::WindowsPair,
    write_stream::write_stream,
    ErrorCode, GoAwayFrame, HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer,
    Http2Data, Http2Rslt, Http2RsltExt, ReqResBuffer, ResetStreamFrame, StreamState, Windows, U31,
  },
  misc::{AsyncBounds, ByteVector, Lease, LeaseMut, Lock, RefCounter, Stream, Uri, _Span},
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
  span: _Span,
  stream_id: U31,
  stream_state: StreamState,
  windows: Windows,
}

impl<HB, HD, S> ServerStream<HB, HD, S> {
  #[inline]
  pub(crate) const fn new(
    hd: HD,
    span: _Span,
    rfi: ReadFramesInit<Method>,
    stream_state: StreamState,
    windows: Windows,
  ) -> Self {
    Self {
      hd,
      hpack_size: rfi.hpack_size,
      is_eos: rfi.is_eos,
      method: rfi.headers_rslt,
      phantom: PhantomData,
      span,
      stream_id: rfi.stream_id,
      stream_state,
      windows,
    }
  }
}

impl<HB, HD, S> ServerStream<HB, HD, S>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, false>>,
      Resource = Http2Data<HB, S, false>,
    > + 'guard,
  S: AsyncBounds + Stream,
{
  /// High-level method that reads all remaining data to build a request.
  //
  // `rrb` won't be cleared because it should have been used earlier when accepting a stream.
  #[inline]
  pub async fn recv_req<'rrb>(
    &mut self,
    rrb: &'rrb mut ReqResBuffer,
  ) -> crate::Result<Http2Rslt<Request<(&'rrb mut ByteVector, &'rrb mut Headers), &'rrb str>>> {
    let _e = self.span._enter();
    _trace!("Receiving request");
    hre_to_hr!(self.hd, |guard| {
      guard
        .read_frames_others(
          &mut self.hpack_size,
          self.is_eos,
          rrb,
          self.stream_id,
          &mut self.stream_state,
          &mut self.windows,
        )
        .await?
    });
    self.stream_state = StreamState::HalfClosedRemote;
    Ok(Http2Rslt::Resource(Request::http2(
      (&mut rrb.body, &mut rrb.headers),
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
  pub async fn send_res<D>(&mut self, res: Response<D>) -> crate::Result<Http2Rslt<()>>
  where
    D: ReqResData,
    D::Body: Lease<[u8]>,
  {
    let _e = self.span._enter();
    _trace!("Sending response");
    let mut body = res.data.body().lease();
    let mut hf = {
      let mut guard = self.hd.lock().await;
      let (hb, _, _, hps, ..) = guard.parts_mut();
      verify_before_send::<false>(
        res.data.headers(),
        &mut hb.hpack_enc,
        &mut hb.hpack_enc_buffer,
        hps,
        (
          HpackStaticRequestHeaders::EMPTY,
          HpackStaticResponseHeaders { status_code: Some(res.status_code) },
        ),
        self.stream_id,
      )?
    };
    hre_to_hr!(self.hd, |guard| {
      let (hb, is_conn_open, _, hps, stream, streams_num, windows) = guard.parts_mut();
      write_stream::<_, false>(
        &mut body,
        &mut hf,
        &mut hb.hpack_enc_buffer,
        hps,
        *is_conn_open,
        stream,
        &mut self.stream_state,
        streams_num,
        &mut WindowsPair::new(windows, &mut self.windows),
      )
      .await?
    });
    self.stream_state = StreamState::Closed;
    Ok(Http2Rslt::Resource(()))
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
