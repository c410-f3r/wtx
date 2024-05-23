use crate::http2::U31;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FrameInit {
  pub(crate) data_len: u32,
  pub(crate) flags: u8,
  pub(crate) stream_id: U31,
  pub(crate) ty: FrameInitTy,
}

create_enum! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq)]
  pub enum FrameInitTy<u8> {
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
  #[inline]
  pub(crate) const fn new(data_len: u32, flags: u8, stream_id: U31, ty: FrameInitTy) -> Self {
    Self { data_len, flags, stream_id, ty }
  }

  #[inline]
  pub(crate) fn from_array(bytes: [u8; 9]) -> (Option<Self>, u32) {
    let [a, b, c, d, e, f, g, h, i] = bytes;
    let data_len = u32::from_be_bytes([0, a, b, c]);
    (
      FrameInitTy::try_from(d).ok().map(|ty| Self {
        data_len,
        flags: e,
        stream_id: U31::from_u32(u32::from_be_bytes([f, g, h, i])),
        ty,
      }),
      data_len,
    )
  }

  #[inline]
  pub(crate) fn bytes(&self) -> [u8; 9] {
    let [_, a, b, c] = self.data_len.to_be_bytes();
    let [e, f, g, h] = self.stream_id.to_be_bytes();
    [a, b, c, self.ty.into(), self.flags, e, f, g, h]
  }

  #[inline]
  pub(crate) fn is_conn_control(&self) -> bool {
    match self.ty {
      FrameInitTy::GoAway | FrameInitTy::Ping | FrameInitTy::Settings => true,
      FrameInitTy::WindowUpdate if self.stream_id.is_zero() => true,
      _ => false,
    }
  }
}
