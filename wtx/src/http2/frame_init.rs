use crate::http2::StreamId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FrameInit {
  pub(crate) flag: u8,
  pub(crate) len: u32,
  pub(crate) stream_id: StreamId,
  pub(crate) ty: FrameHeaderTy,
}

create_enum! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq)]
  pub enum FrameHeaderTy<u8> {
    Data = (0),
    Headers = (1),
    Reset = (3),
    Settings = (4),
    Ping = (6),
    GoAway = (7),
    WindowUpdate = (8),
    Continuation = (9),
  }
}

impl FrameInit {
  pub(crate) fn new(flag: u8, len: u32, stream_id: StreamId, ty: FrameHeaderTy) -> Self {
    Self { flag, len, stream_id, ty }
  }

  pub(crate) fn from_array(bytes: [u8; 9]) -> crate::Result<Self> {
    let [a, b, c, d, e, f, g, h, i] = bytes;
    Ok(Self {
      flag: e,
      len: u32::from_be_bytes([0, a, b, c]),
      stream_id: StreamId::from(u32::from_be_bytes([f, g, h, i])),
      ty: d.try_into()?,
    })
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    let [a, b, c, _] = self.len.to_be_bytes();
    let [e, f, g, h] = self.stream_id.to_be_bytes();
    [a, b, c, self.ty.into(), self.flag, e, f, g, h]
  }
}
