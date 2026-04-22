use crate::{
  http::{
    DEFAULT_INITIAL_WINDOW_LEN, DEFAULT_MAX_CONCURRENT_STREAMS_NUM, DEFAULT_MAX_FRAME_LEN,
    DEFAULT_MAX_HEADERS_LEN, DEFAULT_MAX_HPACK_LEN, MAX_FRAME_LEN_LOWER_BOUND,
    MAX_FRAME_LEN_UPPER_BOUND, u31::U31,
  },
  http2::{Scrp, Sorp, Window, hpack_encoder::HpackEncoder, settings_frame::SettingsFrame},
};
use core::cmp::Ordering;

/// Parameters used when sending data.
#[derive(Debug)]
pub(crate) struct HttpSendParams {
  pub(crate) enable_connect_protocol: u32,
  pub(crate) initial_window_len: U31,
  pub(crate) max_concurrent_streams_num: u32,
  pub(crate) max_frame_len: u32,
  pub(crate) max_headers_len: u32,
  pub(crate) max_hpack_len: u32,
}

impl HttpSendParams {
  pub(crate) fn update(
    &mut self,
    hpack_enc: &mut HpackEncoder,
    scrp: &mut Scrp,
    sf: &SettingsFrame,
    sorp: &mut Sorp,
  ) -> crate::Result<()> {
    if let Some(elem) = sf.enable_connect_protocol() {
      self.enable_connect_protocol = u32::from(elem);
    }
    'update: {
      if let Some(initial_window_size) = sf.initial_window_size() {
        let ordering = initial_window_size.cmp(&self.initial_window_len);
        let (cb, diff): (fn(U31, U31, &mut Window) -> _, U31) = match ordering {
          Ordering::Equal => {
            break 'update;
          }
          Ordering::Greater => (
            |diff, stream_id, window| window.deposit(Some(stream_id), diff.i32()),
            initial_window_size.wrapping_sub(self.initial_window_len),
          ),
          Ordering::Less => (
            |diff, stream_id, window| window.withdrawn(Some(stream_id), diff.i32()),
            self.initial_window_len.wrapping_sub(initial_window_size),
          ),
        };
        for (stream_id, elem) in scrp {
          cb(diff, *stream_id, elem.windows.send_mut())?;
          elem.waker.wake_by_ref();
        }
        for (stream_id, elem) in sorp {
          cb(diff, *stream_id, elem.windows.send_mut())?;
          elem.waker.wake_by_ref();
        }
        self.initial_window_len = initial_window_size;
      }
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
impl Default for HttpSendParams {
  #[inline]
  fn default() -> Self {
    Self {
      enable_connect_protocol: 0,
      initial_window_len: U31::from_u32(DEFAULT_INITIAL_WINDOW_LEN),
      max_hpack_len: DEFAULT_MAX_HPACK_LEN,
      max_headers_len: DEFAULT_MAX_HEADERS_LEN,
      max_frame_len: DEFAULT_MAX_FRAME_LEN,
      max_concurrent_streams_num: DEFAULT_MAX_CONCURRENT_STREAMS_NUM,
    }
  }
}
