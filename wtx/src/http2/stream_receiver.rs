use crate::{
  http::StatusCode,
  http2::{StreamState, Windows},
  misc::_Span,
};

#[derive(Debug)]
pub(crate) struct StreamOverallRecvParams<SB> {
  pub(crate) body_len: u32,
  pub(crate) has_initial_header: bool,
  pub(crate) sb: SB,
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
