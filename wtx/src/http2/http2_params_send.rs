use crate::http2::{
  HpackEncoder, Scrp, SettingsFrame, Sorp, Windows, MAX_FRAME_LEN, MAX_FRAME_LEN_LOWER_BOUND,
  MAX_FRAME_LEN_UPPER_BOUND, MAX_HPACK_LEN, U31,
};

/// Parameters used when sending data.
#[derive(Debug)]
pub(crate) struct Http2ParamsSend {
  pub(crate) enable_connect_protocol: u32,
  pub(crate) initial_window_len: U31,
  pub(crate) max_concurrent_streams_num: u32,
  pub(crate) max_frame_len: u32,
  pub(crate) max_headers_len: u32,
  pub(crate) max_hpack_len: u32,
}

impl Http2ParamsSend {
  pub(crate) fn update<SB>(
    &mut self,
    hpack_enc: &mut HpackEncoder,
    scrp: &mut Scrp,
    sf: &SettingsFrame,
    sorp: &mut Sorp<SB>,
    windows: &mut Windows,
  ) -> crate::Result<()> {
    if let Some(elem) = sf.enable_connect_protocol() {
      self.enable_connect_protocol = u32::from(elem);
    }
    if let Some(elem) = sf.initial_window_size() {
      let diff = elem.wrapping_sub(self.initial_window_len).i32();
      if diff != 0 {
        for (stream_id, elem) in scrp {
          elem.windows.send.deposit(Some(*stream_id), diff)?;
        }
        for (stream_id, elem) in sorp {
          elem.windows.send.deposit(Some(*stream_id), diff)?;
        }
      }
      windows.send.update(elem.i32());
      self.initial_window_len = elem;
    }
    if let Some(elem) = sf.header_table_size() {
      self.max_hpack_len = elem;
      hpack_enc.set_max_dyn_sub_bytes(elem)?;
    }
    if let Some(elem) = sf.max_header_list_size() {
      self.max_headers_len = elem;
    }
    if let Some(elem) = sf.max_frame_size() {
      self.max_frame_len = elem.clamp(MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND);
    }
    if let Some(elem) = sf.max_concurrent_streams() {
      self.max_concurrent_streams_num = elem;
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
      max_hpack_len: MAX_HPACK_LEN,
      max_headers_len: u32::MAX,
      max_frame_len: MAX_FRAME_LEN,
      max_concurrent_streams_num: u32::MAX,
    }
  }
}
