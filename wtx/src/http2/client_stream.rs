use crate::{
  http::{ReqResData, RequestStr, Response},
  http2::{
    http2_rslt::Http2RsltExt,
    misc::{send_go_away, send_reset, verify_before_send},
    window::WindowsPair,
    write_stream::write_stream,
    ErrorCode, GoAwayFrame, HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer,
    Http2Data, Http2Rslt, ReqResBuffer, ResetStreamFrame, StreamState, Windows, U31,
  },
  misc::{AsyncBounds, Lease, LeaseMut, Lock, RefCounter, Stream, Usize, _Span},
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
  windows: Windows,
}

impl<HB, HD, S> ClientStream<HB, HD, S> {
  pub(crate) const fn idle(hd: HD, span: _Span, stream_id: U31, windows: Windows) -> Self {
    Self { phantom: PhantomData, hd, span, stream_id, stream_state: StreamState::Idle, windows }
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
  ) -> crate::Result<Http2Rslt<Response<&'rrb mut ReqResBuffer>>> {
    let _e = self.span._enter();
    _trace!("Receiving response");
    rrb.clear();
    let status_code = hre_to_hr!(self.hd, |guard| {
      rrb.headers.set_max_bytes(*Usize::from(guard.hp().max_cached_headers_len().0));
      guard
        .read_frames_stream(
          &mut *rrb,
          self.stream_id,
          &mut self.stream_state,
          &mut self.windows,
          |hf| hf.hsresh().status_code.ok_or(crate::Error::MissingResponseStatusCode),
        )
        .await?
    });
    self.stream_state = StreamState::Closed;
    Ok(Http2Rslt::Resource(Response::http2(rrb, status_code)))
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
  pub async fn send_req<D>(&mut self, req: RequestStr<'_, D>) -> crate::Result<Http2Rslt<()>>
  where
    D: ReqResData,
    D::Body: Lease<[u8]>,
  {
    let _e = self.span._enter();
    _trace!("Sending request");
    let mut body = req.data.body().lease();
    let mut hf = {
      let mut guard = self.hd.lock().await;
      let (hb, _, _, hps, ..) = guard.parts_mut();
      verify_before_send::<true>(
        req.data.headers(),
        &mut hb.hpack_enc,
        &mut hb.hpack_enc_buffer,
        hps,
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
        self.stream_id,
      )?
    };
    hre_to_hr!(self.hd, |guard| {
      let (hb, is_conn_open, _, hps, stream, streams_num, windows) = guard.parts_mut();
      write_stream::<_, true>(
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
    self.stream_state = StreamState::HalfClosedLocal;
    Ok(Http2Rslt::Resource(()))
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
