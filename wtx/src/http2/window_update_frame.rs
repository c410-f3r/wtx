use crate::http2::{FrameHeaderTy, FrameInit, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WindowUpdateFrame {
  size_increment: U31,
  stream_id: U31,
}

impl WindowUpdateFrame {
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    let [a, b, c, d] = bytes else {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    };
    let size_increment = U31::new(u32::from_be_bytes([*a, *b, *c, *d]));
    if size_increment.is_zero() {
      return Err(crate::http2::ErrorCode::ProtocolError.into());
    }
    Ok(Self { size_increment, stream_id: fi.stream_id })
  }

  pub(crate) fn _bytes(&self) -> [u8; 13] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(4, 0, self.stream_id, FrameHeaderTy::WindowUpdate).bytes();
    let [j, k, l, m] = self.size_increment.to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m]
  }
}
