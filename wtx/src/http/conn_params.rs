#[derive(Clone, Copy, Debug)]
pub(crate) struct ConnParams {
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
  #[inline]
  pub(crate) fn _to_hp(self) -> crate::http2::Http2Params {
    crate::http2::Http2Params::default()
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
    Self {
      _initial_window_len: u32::MAX,
      _max_body_len: 4 * 1024 * 1024,
      _max_concurrent_streams_num: u32::MAX,
      _max_frame_len: 64 * 1024,
      _max_headers_len: 8 * 1024,
      _max_hpack_len: (128 * 1024, 128 * 1024),
      _max_recv_streams_num: u32::MAX,
    }
  }
}
