use crate::{
  http::{ReqResData, RequestStr, StatusCode},
  http2::{
    misc::{check_content_length, send_go_away, send_reset_stream},
    send_msg::send_msg,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2ErrorCode,
    StreamBuffer, StreamOverallRecvParams, StreamState, Windows, U31,
  },
  misc::{ByteVector, Lease, LeaseMut, Lock, RefCounter, Stream, Usize, _Span},
};
use tokio::sync::MutexGuard;

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<HD> {
  hd: HD,
  span: _Span,
  stream_id: U31,
  // Used after the initial sending
  windows: Windows,
}

impl<HD> ClientStream<HD> {
  pub(crate) const fn new(hd: HD, span: _Span, stream_id: U31) -> Self {
    Self { hd, span, stream_id, windows: Windows::new() }
  }
}

impl<HB, HD, S, SB> ClientStream<HD>
where
  HB: LeaseMut<Http2Buffer<SB>>,
  HD: RefCounter,
  for<'guard> HD::Item: Lock<
      Guard<'guard> = MutexGuard<'guard, Http2Data<HB, S, SB, true>>,
      Resource = Http2Data<HB, S, SB, true>,
    > + 'guard,
  S: Stream,
  SB: LeaseMut<StreamBuffer>,
{
  /// Awaits for all remaining data to build a response and then closes the stream.
  ///
  /// Should be called after [`Self::send_req`] is successfully executed.
  #[inline]
  pub async fn recv_res(&mut self, mut sb: SB) -> crate::Result<(SB, StatusCode)> {
    let _e = self.span._enter();
    _trace!("Receiving response");
    {
      let mut guard = self.hd.lock().await;
      let hdpm = guard.parts_mut();
      sb.lease_mut().clear();
      sb.lease_mut().rrb.headers.set_max_bytes(*Usize::from(hdpm.hp.max_headers_len()));
      drop(hdpm.hb.sorp.insert(
        self.stream_id,
        StreamOverallRecvParams {
          body_len: 0,
          content_length_idx: None,
          has_initial_header: false,
          sb,
          span: _Span::_none(),
          status_code: StatusCode::Ok,
          stream_state: StreamState::HalfClosedLocal,
          windows: self.windows,
        },
      ));
    }
    process_higher_operation!(&self.hd, |guard| {
      let mut fun = || {
        let hdpm = guard.parts_mut();
        if hdpm.hb.sorp.get(&self.stream_id).map_or(false, |el| el.stream_state.recv_eos()) {
          if let Some(sorp) = hdpm.hb.sorp.remove(&self.stream_id) {
            if let Some(idx) = sorp.content_length_idx {
              check_content_length(idx, &sorp)?;
            }
            return Ok(Some((sorp.sb, sorp.status_code)));
          }
        }
        Ok(None)
      };
      fun()
    })
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  pub async fn send_go_away(&mut self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    send_go_away(error_code, hdpm.is_conn_open, *hdpm.last_stream_id, hdpm.stream).await
  }

  /// Send Request
  ///
  /// Composes all header frames, data frames and trailer frames that will be sent in a stream.
  ///
  /// Shouldn't be called more than one time.
  #[inline]
  pub async fn send_req<D>(
    &mut self,
    hpack_enc_buffer: &mut ByteVector,
    req: RequestStr<'_, D>,
  ) -> crate::Result<()>
  where
    D: ReqResData,
    D::Body: Lease<[u8]>,
  {
    let _e = self.span._enter();
    _trace!("Sending response");
    let hsreqh = HpackStaticRequestHeaders {
      authority: req.uri.authority().as_bytes(),
      method: Some(req.method),
      path: req.uri.href().as_bytes(),
      protocol: None,
      scheme: req.uri.schema().as_bytes(),
    };
    send_msg::<_, _, _, _, true>(
      req.data.body().lease(),
      &self.hd,
      req.data.headers(),
      hpack_enc_buffer,
      (hsreqh, HpackStaticResponseHeaders::EMPTY),
      self.stream_id,
      |hdpm| {
        if let Some(elem) = hdpm.hb.scrp.remove(&self.stream_id) {
          self.windows = elem.windows;
        }
      },
    )
    .await
  }

  /// Sends a RST_STREAM frame to the peer, which cancels this stream.
  pub async fn send_reset(&mut self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    send_reset_stream(error_code, hdpm.hb, hdpm.stream, self.stream_id).await;
  }
}
