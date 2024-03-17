use crate::{
  http2::{
    FrameHeaderTy, FrameInit, StreamId, ACK_MASK, MAX_FRAME_SIZE_LOWER_BOUND,
    MAX_FRAME_SIZE_UPPER_BOUND, MAX_INITIAL_WINDOW_SIZE,
  },
  misc::{ArrayChunks, Stream, _unlikely_elem},
};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct SettingsFrame {
  enable_connect_protocol: Option<u32>,
  flags: SettingsFrameFlags,
  header_table_size: Option<u32>,
  initial_window_size: Option<u32>,
  len: u8,
  max_concurrent_streams: Option<u32>,
  max_frame_size: Option<u32>,
  max_header_list_size: Option<u32>,
}

impl SettingsFrame {
  pub(crate) fn ack() -> SettingsFrame {
    SettingsFrame { flags: SettingsFrameFlags::ack(), ..SettingsFrame::default() }
  }

  pub(crate) fn flags(&self) -> SettingsFrameFlags {
    self.flags
  }

  pub(crate) fn header_table_size(&self) -> Option<u32> {
    self.header_table_size
  }

  pub(crate) fn initial_window_size(&self) -> Option<u32> {
    self.initial_window_size
  }

  pub(crate) fn is_ack(&self) -> bool {
    self.flags.is_ack()
  }

  pub(crate) fn is_extended_connect_protocol_enabled(&self) -> Option<bool> {
    self.enable_connect_protocol.map(|elem| elem != 0)
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

  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(crate::http2::ErrorCode::ProtocolError.into());
    }

    if SettingsFrameFlags::load(fi.flag).is_ack() {
      if !bytes.is_empty() {
        return Err(crate::http2::ErrorCode::FrameSizeError.into());
      }
      return Ok(SettingsFrame::ack());
    }

    if bytes.len() % 6 != 0 {
      return _unlikely_elem(Err(crate::http2::ErrorCode::FrameSizeError.into()));
    }

    let Self {
      mut enable_connect_protocol,
      flags,
      mut header_table_size,
      mut initial_window_size,
      mut len,
      mut max_concurrent_streams,
      mut max_frame_size,
      mut max_header_list_size,
    } = SettingsFrame::default();

    for [a, b, c, d, e, f] in ArrayChunks::_new(bytes) {
      let Ok(setting) = Setting::new(&[*a, *b, *c, *d, *e, *f]) else {
        continue;
      };
      match setting {
        Setting::EnableConnectProtocol(elem) => match elem {
          0 | 1 => {
            enable_connect_protocol = Some(elem);
          }
          _ => {
            return Err(crate::http2::ErrorCode::ProtocolError.into());
          }
        },
        Setting::HeaderTableSize(elem) => {
          header_table_size = Some(elem);
        }
        Setting::InitialWindowSize(elem) => {
          if elem > MAX_INITIAL_WINDOW_SIZE {
            return Err(crate::http2::ErrorCode::ProtocolError.into());
          } else {
            initial_window_size = Some(elem);
          }
        }
        Setting::MaxConcurrentStreams(elem) => {
          max_concurrent_streams = Some(elem);
        }
        Setting::MaxFrameSize(elem) => {
          if MAX_FRAME_SIZE_LOWER_BOUND <= elem && elem <= MAX_FRAME_SIZE_UPPER_BOUND {
            max_frame_size = Some(elem);
          } else {
            return Err(crate::http2::ErrorCode::ProtocolError.into());
          }
        }
        Setting::MaxHeaderListSize(elem) => {
          max_header_list_size = Some(elem);
        }
      }
    }

    let array = [
      enable_connect_protocol,
      header_table_size,
      initial_window_size,
      max_concurrent_streams,
      max_frame_size,
      max_header_list_size,
    ];
    for _ in array.into_iter().flatten() {
      len = len.wrapping_add(6);
    }

    Ok(Self {
      enable_connect_protocol,
      flags,
      header_table_size,
      initial_window_size,
      len,
      max_concurrent_streams,
      max_frame_size,
      max_header_list_size,
    })
  }

  pub(crate) fn set_enable_connect_protocol(&mut self, elem: Option<u32>) {
    self.enable_connect_protocol = elem;
  }

  pub(crate) fn set_header_table_size(&mut self, elem: Option<u32>) {
    self.header_table_size = elem;
  }

  pub(crate) fn set_initial_window_size(&mut self, elem: Option<u32>) {
    self.initial_window_size = elem;
  }

  pub(crate) fn set_max_concurrent_streams(&mut self, elem: Option<u32>) {
    self.max_concurrent_streams = elem;
  }

  pub(crate) fn set_max_frame_size(&mut self, elem: Option<u32>) {
    if let Some(elem) = elem {
      assert!(MAX_FRAME_SIZE_LOWER_BOUND <= elem && elem <= MAX_FRAME_SIZE_UPPER_BOUND);
    }
    self.max_frame_size = elem;
  }

  pub(crate) fn set_max_header_list_size(&mut self, elem: Option<u32>) {
    self.max_header_list_size = elem;
  }

  pub(crate) async fn write<S>(&self, stream: &mut S) -> crate::Result<()>
  where
    S: Stream,
  {
    fn bytes(ty: u16, value: u32) -> [u8; 6] {
      let [a, b] = ty.to_be_bytes();
      let [c, d, e, f] = value.to_be_bytes();
      [a, b, c, d, e, f]
    }
    let Self {
      enable_connect_protocol,
      flags: _,
      header_table_size,
      initial_window_size,
      len: _,
      max_concurrent_streams,
      max_frame_size,
      max_header_list_size,
    } = self;
    stream
      .write_vectored(&[
        FrameInit::new(
          self.flags().into(),
          self.len.into(),
          StreamId::ZERO,
          FrameHeaderTy::Settings,
        )
        .bytes()
        .as_slice(),
        header_table_size.map(|el| bytes(1, el)).as_ref().map(AsRef::as_ref).unwrap_or_default(),
        max_concurrent_streams
          .map(|el| bytes(3, el))
          .as_ref()
          .map(AsRef::as_ref)
          .unwrap_or_default(),
        initial_window_size.map(|el| bytes(4, el)).as_ref().map(AsRef::as_ref).unwrap_or_default(),
        max_frame_size.map(|el| bytes(5, el)).as_ref().map(AsRef::as_ref).unwrap_or_default(),
        max_header_list_size.map(|el| bytes(6, el)).as_ref().map(AsRef::as_ref).unwrap_or_default(),
        enable_connect_protocol
          .map(|el| bytes(8, el))
          .as_ref()
          .map(AsRef::as_ref)
          .unwrap_or_default(),
      ])
      .await?;
    Ok(())
  }
}

#[derive(Debug)]
enum Setting {
  EnableConnectProtocol(u32),
  HeaderTableSize(u32),
  InitialWindowSize(u32),
  MaxConcurrentStreams(u32),
  MaxFrameSize(u32),
  MaxHeaderListSize(u32),
}

impl Setting {
  pub(crate) fn new(array: &[u8; 6]) -> crate::Result<Setting> {
    let [a, b, c, d, e, f] = array;
    Setting::from_id((u16::from(*a) << 8) | u16::from(*b), u32::from_be_bytes([*c, *d, *e, *f]))
  }

  pub(crate) fn from_id(id: u16, value: u32) -> crate::Result<Setting> {
    Ok(match id {
      1 => Self::HeaderTableSize(value),
      3 => Self::MaxConcurrentStreams(value),
      4 => Self::InitialWindowSize(value),
      5 => Self::MaxFrameSize(value),
      6 => Self::MaxHeaderListSize(value),
      8 => Self::EnableConnectProtocol(value),
      _ => return Err(crate::Error::UnknownSettingFrameTy),
    })
  }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct SettingsFrameFlags(u8);

impl SettingsFrameFlags {
  pub(crate) fn ack() -> SettingsFrameFlags {
    SettingsFrameFlags(ACK_MASK)
  }

  pub(crate) fn empty() -> SettingsFrameFlags {
    SettingsFrameFlags(0)
  }

  pub(crate) fn load(bits: u8) -> SettingsFrameFlags {
    SettingsFrameFlags(bits & ACK_MASK)
  }

  pub(crate) fn is_ack(&self) -> bool {
    self.0 & ACK_MASK == ACK_MASK
  }
}

impl From<SettingsFrameFlags> for u8 {
  #[inline]
  fn from(src: SettingsFrameFlags) -> u8 {
    src.0
  }
}
