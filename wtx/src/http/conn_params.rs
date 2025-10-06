#[derive(Clone, Copy, Debug)]
pub(crate) struct ConnParams {
  pub(crate) _enable_connect_protocol: bool,
  pub(crate) _initial_window_len: u32,
  pub(crate) _max_body_len: u32,
  pub(crate) _max_concurrent_streams_num: u32,
  pub(crate) _max_frame_len: u32,
  pub(crate) _max_headers_len: u32,
  pub(crate) _max_hpack_len: (u32, u32),
  pub(crate) _max_recv_streams_num: u32,
}

#[cfg(feature = "http2")]
impl ConnParams {
  pub(crate) const fn new() -> Self {
    Self {
      _enable_connect_protocol: false,
      _initial_window_len: u32::MAX,
      _max_body_len: 64 * 1024 * 1024,
      _max_concurrent_streams_num: u32::MAX,
      _max_frame_len: 64 * 1024,
      _max_headers_len: 8 * 1024,
      _max_hpack_len: (128 * 1024, 128 * 1024),
      _max_recv_streams_num: u32::MAX,
    }
  }

  pub(crate) const fn _to_hp(self) -> crate::http2::Http2Params {
    crate::http2::Http2Params::with_default_params()
      .set_enable_connect_protocol(self._enable_connect_protocol)
      .set_initial_window_len(self._initial_window_len)
      .set_max_body_len(self._max_body_len)
      .set_max_concurrent_streams_num(self._max_concurrent_streams_num)
      .set_max_frame_len(self._max_frame_len)
      .set_max_headers_len(self._max_headers_len)
      .set_max_hpack_len(self._max_hpack_len)
      .set_max_recv_streams_num(self._max_recv_streams_num)
  }
}

impl Default for ConnParams {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
