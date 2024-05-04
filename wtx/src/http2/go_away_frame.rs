use crate::http2::{ErrorCode, FrameHeaderTy, FrameInit, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct GoAwayFrame {
  error_code: ErrorCode,
  last_stream_id: U31,
}

impl GoAwayFrame {
  pub(crate) fn new(error_code: ErrorCode, last_stream_id: U31) -> Self {
    Self { error_code, last_stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 17] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(8, 0, U31::ZERO, FrameHeaderTy::GoAway).bytes();
    let [j, k, l, m] = self.last_stream_id.to_be_bytes();
    let [n, o, p, q] = u32::from(self.error_code).to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q]
  }

  pub(crate) fn read(data: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(ErrorCode::ProtocolError.into());
    }
    let [a, b, c, d, e, f, g, h, ..] = data else {
      return Err(ErrorCode::FrameSizeError.into());
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*e, *f, *g, *h]).try_into()?,
      last_stream_id: U31::new(u32::from_be_bytes([*a, *b, *c, *d])),
    })
  }
}
