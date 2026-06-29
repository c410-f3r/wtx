use crate::{http::u31::U31, http2::common_flags::CommonFlags};

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
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8] = bytes;
    let data_len = u32::from_be_bytes([0, b0, b1, b2]);
    (
      FrameInitTy::try_from(b3).ok().map(|ty| Self {
        data_len,
        cf: CommonFlags::new(b4),
        stream_id: U31::from_u32(u32::from_be_bytes([b5, b6, b7, b8])),
        ty,
      }),
      data_len,
    )
  }

  pub(crate) const fn bytes(&self) -> [u8; 9] {
    let [_, b1, b2, b3] = self.data_len.to_be_bytes();
    let [b4, b5, b6, b7] = self.stream_id.to_be_bytes();
    [b1, b2, b3, self.ty.byte(), self.cf.byte(), b4, b5, b6, b7]
  }
}
