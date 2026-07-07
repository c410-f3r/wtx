use crate::{
  collections::Vector,
  http::{MsgBufferString, MsgData, Request, StatusCode, u31::U31},
  http2::{
    CommonStream, Http2Inner, Http2RecvStatus, Http2SendStatus,
    hpack_static_headers::{HpackStaticRequestHeaders, HpackStaticResponseHeaders},
    misc::{manage_recurrent_receiving_of_overall_stream, process_higher_operation_err},
    stream_receiver::StreamOverallRecvParams,
    stream_state::StreamState,
    window::Windows,
    write_functions::send_msg,
  },
  misc::{Lease, span::Span},
  stream::StreamWriter,
  sync::Arc,
  tls::TlsMode,
};
use core::{future::poll_fn, pin::pin, task::Waker};

/// Groups the methods used by clients that connect to servers.
#[derive(Debug)]
pub struct ClientStream<SW, TM> {
  inner: Arc<Http2Inner<SW, TM, true>>,
  linger: bool,
  span: Span,
  stream_id: U31,
  // Used after the initial sending
  windows: Windows,
}

impl<SW, TM> ClientStream<SW, TM> {
  pub(crate) const fn new(
    inner: Arc<Http2Inner<SW, TM, true>>,
    linger: bool,
    span: Span,
    stream_id: U31,
  ) -> Self {
    Self { inner, linger, span, stream_id, windows: Windows::new() }
  }
}

impl<SW, TM> ClientStream<SW, TM>
where
  SW: StreamWriter,
  TM: TlsMode,
{
  /// See [`CommonStream`].
  #[inline]
  pub const fn common(&mut self) -> CommonStream<'_, SW, TM, true> {
    let Self { inner, linger, span, stream_id, windows: _ } = self;
    CommonStream { inner, linger: *linger, span, stream_id: *stream_id }
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
  ) -> crate::Result<(Http2RecvStatus<StatusCode, ()>, MsgBufferString)> {
    let Self { inner, linger: _, span, stream_id, windows: _ } = self;
    let _e = span.enter();
    _trace!("Receiving response");
    let mut lock_pin = pin!(inner.hd.lock());
    let rslt = poll_fn(|cx| {
      let mut lock = lock_pin!(cx, inner.hd, lock_pin);
      let hdpm = lock.parts_mut();
      manage_recurrent_receiving_of_overall_stream(
        cx,
        hdpm,
        &inner.is_conn_open,
        *stream_id,
        |_, status_code, _, _| status_code,
      )
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
  pub async fn send_req<MD>(
    &mut self,
    enc_buffer: &mut Vector<u8>,
    req: Request<MD>,
  ) -> crate::Result<Http2SendStatus>
  where
    MD: MsgData,
    MD::Body: Lease<[u8]>,
  {
    let Self { inner, linger: _, span, stream_id, windows } = self;
    let _e = span.enter();
    _trace!("Sending request");
    let uri = req.msg_data.uri();
    send_msg::<_, _, true>(
      req.msg_data.body().lease(),
      enc_buffer,
      req.msg_data.headers(),
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
        drop(hdpm.hb.sorps.insert(
          *stream_id,
          StreamOverallRecvParams {
            body_len: 0,
            content_length: None,
            has_initial_header: false,
            has_one_or_more_data_frames: false,
            is_stream_open: true,
            msg_buffer: MsgBufferString::default(),
            status_code: StatusCode::Ok,
            stream_state: StreamState::HalfClosedLocal,
            // The possible future invocation of this waker by the reading task won't be a problem
            // because users will manually call `recv_res`.
            waker: Waker::noop().clone(),
            windows: *windows,
          },
        ));
      },
    )
    .await
  }
}
