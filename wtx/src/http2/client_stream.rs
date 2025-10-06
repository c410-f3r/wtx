use crate::{
  http::{ReqResBuffer, ReqResData, Request, StatusCode},
  http2::{
    CommonStream, Http2Buffer, Http2Data, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{
      frame_reader_rslt, manage_initial_stream_receiving, manage_recurrent_stream_receiving,
      process_higher_operation_err,
    },
    send_msg::send_msg,
    stream_receiver::StreamOverallRecvParams,
    stream_state::StreamState,
    u31::U31,
    window::Windows,
  },
  misc::{Lease, LeaseMut, UriRef, span::Span},
  stream::StreamWriter,
  sync::{Arc, AtomicBool, Lock, RefCounter},
};
use core::{future::poll_fn, pin::pin, task::Poll};

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<HD> {
  hd: HD,
  is_conn_open: Arc<AtomicBool>,
  span: Span,
  stream_id: U31,
  // Used after the initial sending
  windows: Windows,
}

impl<HD> ClientStream<HD> {
  pub(crate) const fn new(
    hd: HD,
    is_conn_open: Arc<AtomicBool>,
    span: Span,
    stream_id: U31,
  ) -> Self {
    Self { hd, is_conn_open, span, stream_id, windows: Windows::new() }
  }
}

impl<HB, HD, SW> ClientStream<HD>
where
  HB: LeaseMut<Http2Buffer>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, SW, true>>,
  SW: StreamWriter,
{
  /// See [`CommonStream`].
  #[inline]
  pub const fn common(&mut self) -> CommonStream<'_, HD, true> {
    CommonStream {
      hd: &mut self.hd,
      is_conn_open: &self.is_conn_open,
      span: &mut self.span,
      stream_id: self.stream_id,
    }
  }

  /// Receive response
  ///
  /// High-level operation that awaits for the data necessary to build a response and then closes the
  /// stream.
  ///
  /// Returns [`Option::None`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Should be called after [`Self::send_req`] or any other low level methods that send data
  /// are successfully executed. More specifically, should only be called in a half-closed stream
  /// state.
  #[inline]
  pub async fn recv_res(
    &mut self,
    rrb: ReqResBuffer,
  ) -> crate::Result<(Http2RecvStatus<StatusCode, ()>, ReqResBuffer)> {
    let rrb_opt = &mut Some(rrb);
    let Self { hd, is_conn_open, span, stream_id, windows } = self;
    let _e = span.enter();
    _trace!("Receiving response");
    let mut lock_pin = pin!(hd.lock());
    let rslt = poll_fn(|cx| {
      let mut lock = lock_pin!(cx, hd, lock_pin);
      let hdpm = lock.parts_mut();
      if let Some(mut elem) = rrb_opt.take() {
        if !manage_initial_stream_receiving(is_conn_open, &mut elem) {
          frame_reader_rslt(hdpm.frame_reader_error)?;
          return Poll::Ready(Ok((Http2RecvStatus::ClosedConnection, elem)));
        }
        drop(hdpm.hb.sorp.insert(
          *stream_id,
          StreamOverallRecvParams {
            body_len: 0,
            content_length: None,
            has_initial_header: false,
            has_one_or_more_data_frames: false,
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

  /// Send Request
  ///
  /// Sends all data related to a request.
  ///
  /// Returns [`Option::None`] if the network/stream connection has been closed, either locally
  /// or externally.
  ///
  /// Shouldn't be called more than once.
  #[inline]
  pub async fn send_req<RRD>(
    &mut self,
    req: Request<RRD>,
    uri: &UriRef<'_>,
  ) -> crate::Result<Http2SendStatus>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    let _e = self.span.enter();
    _trace!("Sending request");
    send_msg::<_, _, _, true>(
      req.rrd.body().lease(),
      &self.hd,
      req.rrd.headers(),
      (
        HpackStaticRequestHeaders {
          authority: uri.authority(),
          method: Some(req.method),
          path: uri.relative_reference_slash(),
          protocol: None,
          scheme: uri.scheme(),
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
}
