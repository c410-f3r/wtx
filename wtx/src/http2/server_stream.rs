use crate::{
  http::{Method, ReqResBuffer, ReqResData, Response},
  http2::{
    misc::{check_content_length, send_go_away, send_reset_stream},
    send_msg::send_msg,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2ErrorCode,
    StreamControlRecvParams, U31,
  },
  misc::{Lease, LeaseMut, Lock, RefCounter, Stream, _Span},
};

/// Created when a server receives an initial stream.
#[derive(Debug)]
pub struct ServerStream<HD> {
  hd: HD,
  method: Method,
  span: _Span,
  stream_id: U31,
}

impl<HD> ServerStream<HD> {
  #[inline]
  pub(crate) const fn new(hd: HD, method: Method, span: _Span, stream_id: U31) -> Self {
    Self { hd, method, span, stream_id }
  }
}

impl<HB, HD, RRB, S> ServerStream<HD>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, false>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
{
  /// Receive request
  ///
  /// Higher operation that awaits for the data necessary to build a request.
  ///
  /// Shouldn't be called more than once.
  #[inline]
  pub async fn recv_req(&mut self) -> crate::Result<(RRB, Method)> {
    let _e = self.span._enter();
    _trace!("Receiving request");
    process_higher_operation!(
      &self.hd,
      |guard| {
        let rslt = 'rslt: {
          let hdpm = guard.parts_mut();
          if hdpm.hb.sorp.get(&self.stream_id).map_or(false, |el| el.stream_state.recv_eos()) {
            if let Some(sorp) = hdpm.hb.sorp.remove(&self.stream_id) {
              if let Some(idx) = sorp.content_length {
                if let Err(err) = check_content_length(idx, &sorp) {
                  break 'rslt Err(err);
                }
              }
              drop(hdpm.hb.scrp.insert(
                self.stream_id,
                StreamControlRecvParams {
                  span: self.span.clone(),
                  stream_state: sorp.stream_state,
                  windows: sorp.windows,
                },
              ));
              break 'rslt Ok(Some((sorp.rrb, self.method)));
            }
          }
          Ok(None)
        };
        rslt
      },
      |_guard, elem| Ok(elem)
    )
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  #[inline]
  pub async fn send_go_away(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    send_go_away(error_code, hdpm.is_conn_open, *hdpm.last_stream_id, hdpm.stream).await;
  }

  /// Send Response
  ///
  /// Higher operation that sends all data related to a response and then closes the stream.
  ///
  /// Should be called after [`Self::recv_req`] is successfully executed.
  #[inline]
  pub async fn send_res<D>(self, res: Response<D>) -> crate::Result<()>
  where
    D: ReqResData,
    D::Body: Lease<[u8]>,
  {
    let _e = self.span._enter();
    _trace!("Sending response");
    send_msg::<_, _, _, _, false>(
      res.rrd.body().lease(),
      &self.hd,
      res.rrd.headers(),
      (
        HpackStaticRequestHeaders::EMPTY,
        HpackStaticResponseHeaders { status_code: Some(res.status_code) },
      ),
      self.stream_id,
      |hdpm| {
        drop(hdpm.hb.scrp.remove(&self.stream_id));
      },
    )
    .await
  }

  /// Sends a stream reset to the peer, which cancels this stream.
  pub async fn send_reset(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    send_reset_stream(error_code, hdpm.hb, hdpm.stream, self.stream_id).await;
  }
}
