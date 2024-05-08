use crate::http2::{
  HpackEncoder, SettingsFrame, Windows, MAX_CACHED_HEADERS_LEN, MAX_FRAME_LEN,
  MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND, U31,
};

/// Parameters used when sending data.
#[derive(Debug)]
pub(crate) struct Http2ParamsSend {
  pub(crate) enable_connect_protocol: u32,
  pub(crate) initial_window_len: U31,
  pub(crate) max_cached_headers_len: u32,
  pub(crate) max_expanded_headers_len: u32,
  pub(crate) max_frame_len: u32,
  pub(crate) max_streams_num: u32,
}

impl Http2ParamsSend {
  pub(crate) fn update(
    &mut self,
    hpack_enc: &mut HpackEncoder,
    sf: &SettingsFrame,
    windows: &mut Windows,
  ) -> crate::Result<()> {
    if let Some(elem) = sf.enable_connect_protocol() {
      self.enable_connect_protocol = u32::from(elem);
    }
    if let Some(elem) = sf.initial_window_size() {
      windows.send.set(elem.i32())?;
      self.initial_window_len = elem;
    }
    if let Some(elem) = sf.header_table_size() {
      self.max_cached_headers_len = elem;
      hpack_enc.set_max_dyn_sub_bytes(elem)?;
    }
    if let Some(elem) = sf.max_header_list_size() {
      self.max_expanded_headers_len = elem;
    }
    if let Some(elem) = sf.max_frame_size() {
      self.max_frame_len = elem.clamp(MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND);
    }
    if let Some(elem) = sf.max_concurrent_streams() {
      self.max_streams_num = elem;
    }
    Ok(())
  }
}

/// It is not possible to use the same default values of `Http2Params` because, for sending
/// purposes, the default values provided by the RFC must be used until a settings frame
/// is received.
impl Default for Http2ParamsSend {
  fn default() -> Self {
    Self {
      enable_connect_protocol: 0,
      initial_window_len: U31::from_u32(initial_window_len!()),
      max_cached_headers_len: MAX_CACHED_HEADERS_LEN,
      max_expanded_headers_len: u32::MAX,
      max_frame_len: MAX_FRAME_LEN,
      max_streams_num: u32::MAX,
    }
  }
}
