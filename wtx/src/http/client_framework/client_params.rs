#[derive(Debug)]
pub(crate) struct ClientParams {
  pub(crate) _initial_window_len: u32,
  pub(crate) _max_body_len: u32,
  pub(crate) _max_frame_len: u32,
  pub(crate) _max_headers_len: u32,
}

impl Default for ClientParams {
  #[inline]
  fn default() -> Self {
    Self {
      _initial_window_len: 32 * 1024 * 1024,
      _max_body_len: 4 * 1024 * 1024,
      _max_frame_len: 64 * 1024,
      _max_headers_len: 8_192,
    }
  }
}
