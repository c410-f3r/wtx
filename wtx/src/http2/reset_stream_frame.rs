use crate::http2::{ErrorCode, FrameHeaderTy, FrameInit, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ResetStreamFrame {
  error_code: ErrorCode,
  stream_id: U31,
}

impl ResetStreamFrame {
  pub(crate) fn new(error_code: ErrorCode, stream_id: U31) -> crate::Result<Self> {
    if stream_id.is_zero() {
      return Err(ErrorCode::ProtocolError.into());
    }
    Ok(Self { error_code, stream_id })
  }

  pub(crate) fn bytes(&self) -> [u8; 13] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(4, 0, self.stream_id, FrameHeaderTy::Reset).bytes();
    let [j, k, l, m] = u32::from(self.error_code).to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m]
  }

  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    let [a, b, c, d] = bytes else {
      return Err(ErrorCode::FrameSizeError.into());
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*a, *b, *c, *d]).try_into()?,
      stream_id: fi.stream_id,
    })
  }
}
