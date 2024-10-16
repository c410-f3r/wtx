use crate::{
  http::{ReqResBuffer, StatusCode},
  http2::{StreamState, Windows},
};
use core::task::Waker;

/// Parameters used when a stream should only receive control frames like `RST_STREAM` or
/// `WINDOW_UPDATE`.
///
/// Used only by unidirectional streams when they are sending data or when the state is closed.
#[derive(Debug)]
pub(crate) struct StreamControlRecvParams {
  pub(crate) is_stream_open: bool,
  pub(crate) stream_state: StreamState,
  pub(crate) waker: Waker,
  pub(crate) windows: Windows,
}

/// Parameters used when a stream should receive any type of frame, including `HEADER` or
/// `DATA` frames.
///
/// Used by bidirectional or unidirectional streams.
#[derive(Debug)]
pub(crate) struct StreamOverallRecvParams<HE> {
  pub(crate) content_length: Option<usize>,
  pub(crate) body_len: u32,
  pub(crate) has_initial_header: bool,
  pub(crate) hook_element: HE,
  pub(crate) is_stream_open: bool,
  pub(crate) rrb: ReqResBuffer,
  pub(crate) status_code: StatusCode,
  pub(crate) stream_state: StreamState,
  pub(crate) windows: Windows,
  pub(crate) waker: Waker,
}
