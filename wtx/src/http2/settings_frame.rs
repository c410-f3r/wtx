use crate::http2::{
  Http2Error, Http2ErrorCode, MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND,
  common_flags::CommonFlags,
  frame_init::{FrameInit, FrameInitTy},
  misc::protocol_err,
  u31::U31,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SettingsFrame {
  cf: CommonFlags,
  enable_connect_protocol: Option<bool>,
  header_table_size: Option<u32>,
  initial_window_size: Option<U31>,
  len: u8,
  max_concurrent_streams: Option<u32>,
  max_frame_size: Option<u32>,
  max_header_list_size: Option<u32>,
}

impl SettingsFrame {
  pub(crate) const fn ack() -> Self {
    SettingsFrame { cf: CommonFlags::ack(), ..SettingsFrame::empty() }
  }

  pub(crate) const fn empty() -> Self {
    Self {
      cf: CommonFlags::empty(),
      enable_connect_protocol: None,
      header_table_size: None,
      initial_window_size: None,
      len: 0,
      max_concurrent_streams: None,
      max_frame_size: None,
      max_header_list_size: None,
    }
  }

  pub(crate) fn bytes<'buffer>(&self, buffer: &'buffer mut [u8; 45]) -> &'buffer [u8] {
    macro_rules! copy_bytes {
      ($buffer:expr, $bytes:expr, $idx:expr) => {{
        let next_idx = $idx.wrapping_add(6);
        if let Some([a, b, c, d, e, f]) = $buffer.get_mut($idx..next_idx) {
          if let Some([g, h, i, j, k, l]) = $bytes {
            *a = g;
            *b = h;
            *c = i;
            *d = j;
            *e = k;
            *f = l;
            $idx = next_idx;
          }
        }
      }};
    }

    #[inline] // INLINE!
    const fn bytes(ty: u16, value: u32) -> [u8; 6] {
      let [a, b] = ty.to_be_bytes();
      let [c, d, e, f] = value.to_be_bytes();
      [a, b, c, d, e, f]
    }

    {
      let fi = FrameInit::new(self.cf, self.len.into(), U31::ZERO, FrameInitTy::Settings);
      let [a, b, c, d, e, f, g, h, i, ..] = buffer;
      let [j, k, l, m, n, o, p, q, r] = fi.bytes();
      *a = j;
      *b = k;
      *c = l;
      *d = m;
      *e = n;
      *f = o;
      *g = p;
      *h = q;
      *i = r;
    }
    let Self {
      cf: _,
      enable_connect_protocol,
      header_table_size,
      initial_window_size,
      len: _,
      max_concurrent_streams,
      max_frame_size,
      max_header_list_size,
    } = self;
    let mut idx: usize = 9;
    copy_bytes!(buffer, header_table_size.map(|el| bytes(1, el)), idx);
    copy_bytes!(buffer, max_concurrent_streams.map(|el| bytes(3, el)), idx);
    copy_bytes!(buffer, initial_window_size.map(|el| bytes(4, el.u32())), idx);
    copy_bytes!(buffer, max_frame_size.map(|el| bytes(5, el)), idx);
    copy_bytes!(buffer, max_header_list_size.map(|el| bytes(6, el)), idx);
    copy_bytes!(buffer, enable_connect_protocol.map(|el| bytes(8, u32::from(el))), idx);
    buffer.get(..idx).unwrap_or_default()
  }

  pub(crate) fn enable_connect_protocol(&self) -> Option<bool> {
    self.enable_connect_protocol
  }

  pub(crate) fn has_ack(&self) -> bool {
    self.cf.has_ack()
  }

  pub(crate) fn header_table_size(&self) -> Option<u32> {
    self.header_table_size
  }

  pub(crate) fn initial_window_size(&self) -> Option<U31> {
    self.initial_window_size
  }

  pub(crate) fn max_concurrent_streams(&self) -> Option<u32> {
    self.max_concurrent_streams
  }

  pub(crate) fn max_frame_size(&self) -> Option<u32> {
    self.max_frame_size
  }

  pub(crate) fn max_header_list_size(&self) -> Option<u32> {
    self.max_header_list_size
  }

  pub(crate) fn read(bytes: &[u8], mut fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(protocol_err(Http2Error::InvalidSettingsFrameNonZeroId));
    }

    fi.cf.only_ack();
    let mut settings_frame = SettingsFrame { cf: fi.cf, ..Self::empty() };

    if settings_frame.cf.has_ack() {
      if !bytes.is_empty() {
        return Err(crate::Error::Http2ErrorGoAway(
          Http2ErrorCode::FrameSizeError,
          Http2Error::InvalidSettingsFrameNonEmptyAck,
        ));
      }
      return Ok(settings_frame);
    }

    if bytes.len() % 6 != 0 {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidSettingsFrameLength,
      ));
    }

    let Self {
      cf: _,
      enable_connect_protocol,
      header_table_size,
      initial_window_size,
      len,
      max_concurrent_streams,
      max_frame_size,
      max_header_list_size,
    } = &mut settings_frame;

    for [a, b, c, d, e, f] in bytes.as_chunks().0 {
      let Ok(setting) = Setting::new([*a, *b, *c, *d, *e, *f]) else {
        continue;
      };
      match setting {
        Setting::EnableConnectProtocol(elem) => {
          *enable_connect_protocol = Some(elem);
        }
        Setting::HeaderTableSize(elem) => {
          *header_table_size = Some(elem);
        }
        Setting::InitialWindowSize(elem) => {
          if elem > U31::MAX {
            return Err(protocol_err(Http2Error::OutOfBoundsWindowSize));
          }
          *initial_window_size = Some(U31::from_u32(elem));
        }
        Setting::MaxConcurrentStreams(elem) => {
          *max_concurrent_streams = Some(elem);
        }
        Setting::MaxFrameSize(elem) => {
          if (MAX_FRAME_LEN_LOWER_BOUND..=MAX_FRAME_LEN_UPPER_BOUND).contains(&elem) {
            *max_frame_size = Some(elem);
          } else {
            return Err(protocol_err(Http2Error::OutOfBoundsMaxFrameSize));
          }
        }
        Setting::MaxHeaderListSize(elem) => {
          *max_header_list_size = Some(elem);
        }
      }
    }

    if enable_connect_protocol.is_some() {
      *len = len.wrapping_add(6);
    }
    if header_table_size.is_some() {
      *len = len.wrapping_add(6);
    }
    if initial_window_size.is_some() {
      *len = len.wrapping_add(6);
    }
    if max_concurrent_streams.is_some() {
      *len = len.wrapping_add(6);
    }
    if max_frame_size.is_some() {
      *len = len.wrapping_add(6);
    }
    if max_header_list_size.is_some() {
      *len = len.wrapping_add(6);
    }

    Ok(settings_frame)
  }

  pub(crate) fn set_enable_connect_protocol(&mut self, elem: Option<bool>) {
    Self::update_len(&mut self.len, self.enable_connect_protocol, elem);
    self.enable_connect_protocol = elem;
  }

  pub(crate) fn set_header_table_size(&mut self, elem: Option<u32>) {
    Self::update_len(&mut self.len, self.header_table_size, elem);
    self.header_table_size = elem;
  }

  pub(crate) fn set_initial_window_size(&mut self, elem: Option<U31>) {
    Self::update_len(&mut self.len, self.initial_window_size, elem);
    self.initial_window_size = elem;
  }

  pub(crate) fn set_max_concurrent_streams(&mut self, elem: Option<u32>) {
    Self::update_len(&mut self.len, self.max_concurrent_streams, elem);
    self.max_concurrent_streams = elem;
  }

  pub(crate) fn set_max_frame_size(&mut self, elem: Option<u32>) {
    Self::update_len(&mut self.len, self.max_frame_size, elem);
    self.max_frame_size =
      elem.map(|val| val.clamp(MAX_FRAME_LEN_LOWER_BOUND, MAX_FRAME_LEN_UPPER_BOUND));
  }

  pub(crate) fn set_max_header_list_size(&mut self, elem: Option<u32>) {
    Self::update_len(&mut self.len, self.max_header_list_size, elem);
    self.max_header_list_size = elem;
  }

  fn update_len<T>(len: &mut u8, a: Option<T>, b: Option<T>) {
    match (a, b) {
      (None, Some(_)) => {
        *len = len.wrapping_add(6);
      }
      (Some(_), None) => {
        *len = len.wrapping_sub(6);
      }
      _ => {}
    }
  }
}

#[derive(Debug)]
enum Setting {
  EnableConnectProtocol(bool),
  HeaderTableSize(u32),
  InitialWindowSize(u32),
  MaxConcurrentStreams(u32),
  MaxFrameSize(u32),
  MaxHeaderListSize(u32),
}

impl Setting {
  pub(crate) fn new(array: [u8; 6]) -> crate::Result<Setting> {
    let [a, b, c, d, e, f] = array;
    Setting::from_id((u16::from(a) << 8) | u16::from(b), u32::from_be_bytes([c, d, e, f]))
  }

  pub(crate) const fn from_id(id: u16, value: u32) -> crate::Result<Setting> {
    Ok(match id {
      1 => Self::HeaderTableSize(value),
      3 => Self::MaxConcurrentStreams(value),
      4 => Self::InitialWindowSize(value),
      5 => Self::MaxFrameSize(value),
      6 => Self::MaxHeaderListSize(value),
      8 => Self::EnableConnectProtocol(value != 0),
      _ => return Err(protocol_err(Http2Error::UnknownSettingFrameTy)),
    })
  }
}
