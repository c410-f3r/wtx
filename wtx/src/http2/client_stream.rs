use crate::{
  http::{ReqResBuffer, ReqResData, ReqUri, Request, StatusCode},
  http2::{
    misc::{
      frame_reader_rslt, manage_initial_stream_receiving, manage_recurrent_stream_receiving,
      process_higher_operation_err, send_go_away, send_reset_stream,
    },
    send_msg::send_msg,
    HpackStaticRequestHeaders, HpackStaticResponseHeaders, Http2Buffer, Http2Data, Http2ErrorCode,
    StreamOverallRecvParams, StreamState, Windows, U31,
  },
  misc::{Lease, LeaseMut, Lock, RefCounter, StreamWriter, _Span},
};
use alloc::sync::Arc;
use core::{
  future::{poll_fn, Future},
  pin::pin,
  sync::atomic::AtomicBool,
  task::Poll,
};

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<HD> {
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
  span: _Span,
  stream_id: U31,
  // Used after the initial sending
  windows: Windows,
}

impl<HD> ClientStream<HD> {
  #[inline]
  pub(crate) const fn new(
    hd: HD,
    is_conn_open: Arc<AtomicBool>,
    span: _Span,
    stream_id: U31,
  ) -> Self {
    Self { hd, is_conn_open, span, stream_id, windows: Windows::new() }
  }
}

impl<HB, HD, RRB, SW> ClientStream<HD>
where
  HB: LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, SW, true>>,
  RRB: LeaseMut<ReqResBuffer>,
  SW: StreamWriter,
{
  /// Receive response
  ///
  /// Higher operation that awaits for the data necessary to build a response and then closes the
  /// stream.
  ///
  /// Returns [`Option::None`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Should be called after [`Self::send_req`] is successfully executed.
  #[inline]
  pub async fn recv_res(&mut self, rrb: RRB) -> crate::Result<(RRB, Option<StatusCode>)> {
    let rrb_opt = &mut Some(rrb);
    let Self { hd, is_conn_open, span, stream_id, windows } = self;
    let _e = span._enter();
    _trace!("Receiving response");
    let mut lock_pin = pin!(hd.lock());
    let rslt = poll_fn(|cx| {
      let mut lock = lock_pin!(cx, hd, lock_pin);
      let hdpm = lock.parts_mut();
      if let Some(mut elem) = rrb_opt.take() {
        if !manage_initial_stream_receiving(is_conn_open, &mut elem) {
          frame_reader_rslt(hdpm.frame_reader_error)?;
          return Poll::Ready(Ok((elem, None)));
        }
        drop(hdpm.hb.sorp.insert(
          *stream_id,
          StreamOverallRecvParams {
            body_len: 0,
            content_length: None,
            has_initial_header: false,
            is_stream_open: true,
            rrb: elem,
            status_code: StatusCode::Ok,
            stream_state: StreamState::HalfClosedLocal,
            waker: cx.waker().clone(),
            windows: *windows,
          },
        ));
        Poll::Pending
      } else {
        manage_recurrent_stream_receiving(cx, hdpm, is_conn_open, *stream_id, |_, _, sorp| {
          sorp.status_code
        })
      }
    })
    .await;
    if let Err(err) = &rslt {
      process_higher_operation_err(err, hd).await;
    }
    rslt
  }

  /// Sends a GOAWAY frame to the peer, which cancels the connection and consequently all ongoing
  /// streams.
  #[inline]
  pub async fn send_go_away(self, error_code: Http2ErrorCode) {
    send_go_away(error_code, &mut self.hd.lock().await.parts_mut()).await;
  }

  /// Send Request
  ///
  /// Higher operation that sends all data related to a request.
  ///
  /// Returns [`Option::None`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Shouldn't be called more than once.
  #[inline]
  pub async fn send_req<RRD>(
    &mut self,
    req: Request<RRD>,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<Option<()>>
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
          path: uri.relative_reference_slash().as_bytes(),
          protocol: None,
          scheme: uri.scheme().as_bytes(),
        },
        HpackStaticResponseHeaders::EMPTY,
      ),
      &self.is_conn_open,
      self.stream_id,
      |hdpm| {
        if let Some(scrp) = hdpm.hb.scrp.remove(&self.stream_id) {
          self.windows = scrp.windows;
        }
      },
    )
    .await
  }

  /// Sends a `RST_STREAM` frame to the peer, which cancels this stream.
  #[inline]
  pub async fn send_reset(self, error_code: Http2ErrorCode) {
    let mut guard = self.hd.lock().await;
    let hdpm = guard.parts_mut();
    let _ = send_reset_stream(
      error_code,
      &mut hdpm.hb.scrp,
      &mut hdpm.hb.sorp,
      hdpm.stream_writer,
      self.stream_id,
    )
    .await;
  }
}
