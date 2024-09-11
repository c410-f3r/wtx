#[derive(Clone, Copy, Debug)]
pub(crate) struct ConnParams {
  pub(crate) initial_window_len: u32,
  pub(crate) max_body_len: u32,
  pub(crate) max_concurrent_streams_num: u32,
  pub(crate) max_frame_len: u32,
  pub(crate) max_headers_len: u32,
  pub(crate) max_hpack_len: (u32, u32),
  pub(crate) max_recv_streams_num: u32,
}

#[cfg(feature = "http2")]
impl ConnParams {
  #[inline]
  pub(crate) fn to_hp(self) -> crate::http2::Http2Params {
    crate::http2::Http2Params::default()
      .set_initial_window_len(self.initial_window_len)
      .set_max_body_len(self.max_body_len)
      .set_max_concurrent_streams_num(self.max_concurrent_streams_num)
      .set_max_frame_len(self.max_frame_len)
      .set_max_headers_len(self.max_headers_len)
      .set_max_hpack_len(self.max_hpack_len)
      .set_max_recv_streams_num(self.max_recv_streams_num)
  }
}

impl Default for ConnParams {
  #[inline]
  fn default() -> Self {
    Self {
      initial_window_len: u32::MAX,
      max_body_len: 4 * 1024 * 1024,
      max_concurrent_streams_num: u32::MAX,
      max_frame_len: 64 * 1024,
      max_headers_len: 8 * 1024,
      max_hpack_len: (128 * 1024, 128 * 1024),
      max_recv_streams_num: u32::MAX,
    }
  }
}
