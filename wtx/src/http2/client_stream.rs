use crate::{
  collection::Vector,
  http::{ReqResBuffer, ReqResData, Request, StatusCode},
  http2::{
    CommonStream, Http2Buffer, Http2Inner, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{
      connection_state, frame_reader_rslt, manage_recurrent_receiving_of_overall_stream,
      process_higher_operation_err,
    },
    stream_receiver::StreamOverallRecvParams,
    stream_state::StreamState,
    u31::U31,
    window::Windows,
    write_functions::send_msg,
  },
  misc::{Lease, LeaseMut, span::Span},
  stream::StreamWriter,
  sync::Arc,
};
use core::{future::poll_fn, pin::pin, task::Poll};

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<HB, SW> {
  inner: Arc<Http2Inner<HB, SW, true>>,
  span: Span,
  stream_id: U31,
  // Used after the initial sending
  windows: Windows,
}

impl<HB, SW> ClientStream<HB, SW> {
  pub(crate) const fn new(
    inner: Arc<Http2Inner<HB, SW, true>>,
    span: Span,
    stream_id: U31,
  ) -> Self {
    Self { inner, span, stream_id, windows: Windows::new() }
  }
}

impl<HB, SW> ClientStream<HB, SW>
where
  HB: LeaseMut<Http2Buffer>,
  SW: StreamWriter,
{
  /// See [`CommonStream`].
  #[inline]
  pub const fn common(&mut self) -> CommonStream<'_, HB, SW, true> {
    let Self { inner, span, stream_id, windows: _ } = self;
    CommonStream { inner, span, stream_id: *stream_id }
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
    mut rrb: ReqResBuffer,
  ) -> crate::Result<(Http2RecvStatus<StatusCode, ()>, ReqResBuffer)> {
    rrb.clear();
    let Self { inner, span, stream_id, windows } = self;
    let rrb_opt = &mut Some(rrb);
    let _e = span.enter();
    _trace!("Receiving response");
    let mut lock_pin = pin!(inner.hd.lock());
    let rslt = poll_fn(|cx| {
      let mut lock = lock_pin!(cx, inner.hd, lock_pin);
      let hdpm = lock.parts_mut();
      if let Some(elem) = rrb_opt.take() {
        if connection_state(&inner.is_conn_open).is_closed() {
          frame_reader_rslt(hdpm.frame_reader_error)?;
          return Poll::Ready(Ok((Http2RecvStatus::ClosedConnection, elem)));
        }
        drop(hdpm.hb.sorps.insert(
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
        manage_recurrent_receiving_of_overall_stream(
          cx,
          hdpm,
          &inner.is_conn_open,
          *stream_id,
          |_, status_code, _, _| status_code,
        )
      }
    })
    .await;
    if let Err(err) = &rslt {
      process_higher_operation_err(err, inner).await;
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
    enc_buffer: &mut Vector<u8>,
    req: Request<RRD>,
  ) -> crate::Result<Http2SendStatus>
  where
    RRD: ReqResData,
    RRD::Body: Lease<[u8]>,
  {
    let Self { inner, span, stream_id, windows } = self;
    let _e = span.enter();
    _trace!("Sending request");
    let uri = req.rrd.uri();
    send_msg::<_, _, true>(
      req.rrd.body().lease(),
      enc_buffer,
      req.rrd.headers(),
      inner,
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
      *stream_id,
      |hdpm| {
        if let Some(scrp) = hdpm.hb.scrps.remove(stream_id) {
          *windows = scrp.windows;
        }
      },
    )
    .await
  }
}
