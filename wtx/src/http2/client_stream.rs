use crate::{
  http::{ReqResBuffer, ReqResData, ReqUri, Request, StatusCode},
  http2::{
    misc::{check_content_length, send_reset_stream},
    send_msg::send_msg,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2ErrorCode,
    StreamOverallRecvParams, StreamState, Windows, U31,
  },
  misc::{Lease, LeaseMut, Lock, RefCounter, Stream, Usize, _Span},
};

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

impl<HB, HD, RRB, S> ClientStream<HD>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, true>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  /// Receive response
  ///
  /// Higher operation that awaits for the data necessary to build a response and then closes the
  /// stream.
  ///
  /// Should be called after [`Self::send_req`] is successfully executed.
  #[inline]
  pub async fn recv_res(self, mut rrb: RRB) -> crate::Result<(RRB, StatusCode)> {
    let _e = self.span._enter();
    _trace!("Receiving response");
    {
      let mut guard = self.hd.lock().await;
      let hdpm = guard.parts_mut();
      rrb.lease_mut().clear();
      rrb.lease_mut().headers_mut().set_max_bytes(*Usize::from(hdpm.hp.max_headers_len()));
      drop(hdpm.hb.sorp.insert(
        self.stream_id,
        StreamOverallRecvParams {
          body_len: 0,
          content_length_idx: None,
          has_initial_header: false,
          rrb,
          span: _Span::_none(),
          status_code: StatusCode::Ok,
          stream_state: StreamState::HalfClosedLocal,
          windows: self.windows,
        },
      ));
    }
    process_higher_operation!(
      &self.hd,
      |guard| {
        let rslt = 'rslt: {
          let hdpm = guard.parts_mut();
          if hdpm.hb.sorp.get(&self.stream_id).map_or(false, |el| el.stream_state.recv_eos()) {
            if let Some(sorp) = hdpm.hb.sorp.remove(&self.stream_id) {
              if let Some(idx) = sorp.content_length_idx {
                if let Err(err) = check_content_length(idx, &sorp) {
                  break 'rslt Err(err);
                }
              }
              break 'rslt Ok(Some((sorp.rrb, sorp.status_code)));
            }
          }
          Ok(None)
        };
        rslt
      },
      |_guard, elem| Ok(elem)
    )
  }

  /// Send Request
  ///
  /// Higher operation that sends all data related to a request.
  ///
  /// Shouldn't be called more than once.
  #[inline]
  pub async fn send_req<RRD>(
    &mut self,
    req: Request<RRD>,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<()>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    let _e = self.span._enter();
    _trace!("Sending response");
    let uri = match req_uri.into() {
      ReqUri::Data => &req.rrd.uri(),
      ReqUri::Param(elem) => elem,
    };
    send_msg::<_, _, _, _, true>(
      req.rrd.body().lease(),
      &self.hd,
      req.rrd.headers(),
      (
        HpackStaticRequestHeaders {
          authority: uri.authority().as_bytes(),
          method: Some(req.method),
          path: uri.href().as_bytes(),
          protocol: None,
          scheme: uri.schema().as_bytes(),
        },
        HpackStaticResponseHeaders::EMPTY,
      ),
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
  pub async fn send_reset(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    send_reset_stream(error_code, hdpm.hb, hdpm.stream, self.stream_id).await;
  }
}
