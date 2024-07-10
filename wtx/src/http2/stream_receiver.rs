use crate::{
  http::StatusCode,
  http2::{StreamState, Windows},
  misc::_Span,
};

#[derive(Debug)]
pub(crate) struct StreamOverallRecvParams<RRB> {
  pub(crate) content_length_idx: Option<usize>,
  pub(crate) body_len: u32,
  pub(crate) has_initial_header: bool,
  pub(crate) rrb: RRB,
  pub(crate) span: _Span,
  pub(crate) status_code: StatusCode,
  pub(crate) stream_state: StreamState,
  pub(crate) windows: Windows,
}

#[derive(Debug)]
pub(crate) struct StreamControlRecvParams {
  pub(crate) span: _Span,
  pub(crate) stream_state: StreamState,
  pub(crate) windows: Windows,
}
