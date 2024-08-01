#[derive(Debug)]
pub(crate) struct ClientParams {
  pub(crate) initial_window_len: u32,
  pub(crate) max_body_len: u32,
  pub(crate) max_frame_len: u32,
  pub(crate) max_headers_len: u32,
}

impl Default for ClientParams {
  #[inline]
  fn default() -> Self {
    Self {
      initial_window_len: 32 * 1024 * 1024,
      max_body_len: 4 * 1024 * 1024,
      max_frame_len: 64 * 1024,
      max_headers_len: 8_192,
    }
  }
}
