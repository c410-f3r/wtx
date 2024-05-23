use crate::http2::{FrameInit, FrameInitTy, Http2Error, Http2ErrorCode, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WindowUpdateFrame {
  size_increment: U31,
  stream_id: U31,
}

impl WindowUpdateFrame {
  #[inline]
  pub(crate) fn new(size_increment: U31, stream_id: U31) -> crate::Result<Self> {
    if size_increment.is_zero() {
      return Err(crate::Error::http2_go_away_generic(
        Http2Error::InvalidWindowUpdateZeroIncrement,
      ));
    }
    Ok(Self { size_increment, stream_id })
  }

  #[inline]
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    let [a, b, c, d] = bytes else {
      return Err(crate::Error::http2_go_away(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidWindowUpdateFrameBytes,
      ));
    };
    let size_increment = U31::from_u32(u32::from_be_bytes([*a, *b, *c, *d]));
    Self::new(size_increment, fi.stream_id)
  }

  #[inline]
  pub(crate) fn bytes(&self) -> [u8; 13] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(4, 0, self.stream_id, FrameInitTy::WindowUpdate).bytes();
    let [j, k, l, m] = self.size_increment.to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m]
  }

  #[inline]
  pub(crate) fn size_increment(&self) -> U31 {
    self.size_increment
  }
}
