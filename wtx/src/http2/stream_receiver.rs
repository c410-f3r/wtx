use crate::{
  http::StatusCode,
  http2::{StreamState, Windows},
};
use core::task::Waker;

/// Parameters used when a stream should receive any type of frame, including `HEADER` or
/// `DATA` frames.
#[derive(Debug)]
pub(crate) struct StreamOverallRecvParams<RRB> {
  pub(crate) content_length: Option<usize>,
  pub(crate) body_len: u32,
  pub(crate) has_initial_header: bool,
  pub(crate) is_stream_open: bool,
  pub(crate) rrb: RRB,
  pub(crate) status_code: StatusCode,
  pub(crate) stream_state: StreamState,
  pub(crate) windows: Windows,
  pub(crate) waker: Waker,
}

/// Parameters used when a stream should only receive control frames like `RST_STREAM` or
/// `WINDOW_UPDATE`. This can happen when the stream finishes or when the stream is sending data.
#[derive(Debug)]
pub(crate) struct StreamControlRecvParams {
  pub(crate) is_stream_open: bool,
  pub(crate) stream_state: StreamState,
  pub(crate) waker: Waker,
  pub(crate) windows: Windows,
}
