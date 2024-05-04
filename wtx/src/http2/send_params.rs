use crate::http2::{
  HpackEncoder, SettingsFrame, FRAME_LEN_LOWER_BOUND, FRAME_LEN_UPPER_BOUND, U31,
};

/// Parameters used when sending data.
#[derive(Debug)]
pub(crate) struct SendParams {
  pub(crate) enable_connect_protocol: u32,
  pub(crate) initial_window_len: u32,
  pub(crate) max_cached_headers_len: u32,
  pub(crate) max_expanded_headers_len: u32,
  pub(crate) max_frame_len: u32,
  pub(crate) max_streams_num: u32,
}

impl SendParams {
  pub(crate) fn update(
    &mut self,
    hpack_enc: &mut HpackEncoder,
    sf: &SettingsFrame,
  ) -> crate::Result<()> {
    if let Some(elem) = sf.enable_connect_protocol() {
      self.enable_connect_protocol = u32::from(elem);
    }
    if let Some(elem) = sf.initial_window_size() {
      self.initial_window_len = elem.clamp(0, U31::MAX.u32());
    }
    if let Some(elem) = sf.header_table_size() {
      self.max_cached_headers_len = elem;
      hpack_enc.set_max_dyn_sub_bytes(elem)?;
    }
    if let Some(elem) = sf.max_header_list_size() {
      self.max_expanded_headers_len = elem;
    }
    if let Some(elem) = sf.max_frame_size() {
      self.max_frame_len = elem.clamp(FRAME_LEN_LOWER_BOUND, FRAME_LEN_UPPER_BOUND);
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
impl Default for SendParams {
  fn default() -> Self {
    Self {
      enable_connect_protocol: 0,
      initial_window_len: 65_535,
      max_cached_headers_len: 4_096,
      max_expanded_headers_len: u32::MAX,
      max_frame_len: 16_384,
      max_streams_num: u32::MAX,
    }
  }
}
