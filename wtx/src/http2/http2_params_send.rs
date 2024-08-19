use crate::http2::{
  HpackEncoder, Scrp, SettingsFrame, Sorp, MAX_FRAME_LEN, MAX_FRAME_LEN_LOWER_BOUND,
  MAX_FRAME_LEN_UPPER_BOUND, MAX_HPACK_LEN, U31,
};
use core::cmp::Ordering;

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
  pub(crate) fn update<RRB>(
    &mut self,
    hpack_enc: &mut HpackEncoder,
    scrp: &mut Scrp,
    sf: &SettingsFrame,
    sorp: &mut Sorp<RRB>,
  ) -> crate::Result<()> {
    if let Some(elem) = sf.enable_connect_protocol() {
      self.enable_connect_protocol = u32::from(elem);
    }
    if let Some(initial_window_size) = sf.initial_window_size() {
      match initial_window_size.cmp(&self.initial_window_len) {
        Ordering::Equal => {}
        Ordering::Greater => {
          let inc = initial_window_size.wrapping_sub(self.initial_window_len);
          for (stream_id, elem) in scrp {
            elem.windows.send.deposit(Some(*stream_id), inc.i32())?;
            elem.waker.wake_by_ref();
          }
          for (stream_id, elem) in sorp {
            elem.windows.send.deposit(Some(*stream_id), inc.i32())?;
            elem.waker.wake_by_ref();
          }
        }
        Ordering::Less => {
          let dec = self.initial_window_len.wrapping_sub(initial_window_size);
          for (stream_id, elem) in scrp {
            elem.windows.send.withdrawn(Some(*stream_id), dec.i32())?;
            elem.waker.wake_by_ref();
          }
          for (stream_id, elem) in sorp {
            elem.windows.send.withdrawn(Some(*stream_id), dec.i32())?;
            elem.waker.wake_by_ref();
          }
        }
      }
      self.initial_window_len = initial_window_size;
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
