use crate::http2::{
  Http2Error, Http2ErrorCode,
  common_flags::CommonFlags,
  frame_init::{FrameInit, FrameInitTy},
  misc::protocol_err,
  u31::U31,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WindowUpdateFrame {
  size_increment: U31,
  stream_id: U31,
}

impl WindowUpdateFrame {
  pub(crate) const fn new(size_increment: U31, stream_id: U31) -> crate::Result<Self> {
    if size_increment.is_zero() {
      return Err(protocol_err(Http2Error::InvalidWindowUpdateZeroIncrement));
    }
    Ok(Self { size_increment, stream_id })
  }

  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    let [a, b, c, d] = bytes else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidWindowUpdateFrameBytes,
      ));
    };
    let size_increment = U31::from_u32(u32::from_be_bytes([*a, *b, *c, *d]));
    if size_increment > U31::MAX {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidWindowUpdateSize,
      ));
    }
    Self::new(size_increment, fi.stream_id)
  }

  pub(crate) const fn bytes(&self) -> [u8; 13] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(CommonFlags::empty(), 4, self.stream_id, FrameInitTy::WindowUpdate).bytes();
    let [j, k, l, m] = self.size_increment.to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m]
  }

  pub(crate) const fn size_increment(&self) -> U31 {
    self.size_increment
  }
}
