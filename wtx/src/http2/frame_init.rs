use crate::http2::{common_flags::CommonFlags, u31::U31};

create_enum! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq)]
  pub(crate) enum FrameInitTy<u8> {
    Data = (0),
    Headers = (1),
    Priority = (2),
    Reset = (3),
    Settings = (4),
    PushPromise = (5),
    Ping = (6),
    GoAway = (7),
    WindowUpdate = (8),
    Continuation = (9),
  }
}

impl FrameInitTy {
  pub(crate) const fn byte(self) -> u8 {
    match self {
      Self::Data => 0,
      Self::Headers => 1,
      Self::Priority => 2,
      Self::Reset => 3,
      Self::Settings => 4,
      Self::PushPromise => 5,
      Self::Ping => 6,
      Self::GoAway => 7,
      Self::WindowUpdate => 8,
      Self::Continuation => 9,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FrameInit {
  pub(crate) cf: CommonFlags,
  pub(crate) data_len: u32,
  pub(crate) stream_id: U31,
  pub(crate) ty: FrameInitTy,
}

impl FrameInit {
  pub(crate) const fn new(cf: CommonFlags, data_len: u32, stream_id: U31, ty: FrameInitTy) -> Self {
    Self { cf, data_len, stream_id, ty }
  }

  pub(crate) fn from_array(bytes: [u8; 9]) -> (Option<Self>, u32) {
    let [a, b, c, d, e, f, g, h, i] = bytes;
    let data_len = u32::from_be_bytes([0, a, b, c]);
    (
      FrameInitTy::try_from(d).ok().map(|ty| Self {
        data_len,
        cf: CommonFlags::new(e),
        stream_id: U31::from_u32(u32::from_be_bytes([f, g, h, i])),
        ty,
      }),
      data_len,
    )
  }

  pub(crate) const fn bytes(&self) -> [u8; 9] {
    let [_, a, b, c] = self.data_len.to_be_bytes();
    let [e, f, g, h] = self.stream_id.to_be_bytes();
    [a, b, c, self.ty.byte(), self.cf.byte(), e, f, g, h]
  }
}
