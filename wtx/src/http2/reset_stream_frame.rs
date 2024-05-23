use crate::http2::{FrameInit, FrameInitTy, Http2Error, Http2ErrorCode, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ResetStreamFrame {
  error_code: Http2ErrorCode,
  stream_id: U31,
}

impl ResetStreamFrame {
  pub(crate) fn new(error_code: Http2ErrorCode, stream_id: U31) -> Self {
    Self { error_code, stream_id }
  }

  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidResetStreamFrameBytes));
    }
    let [a, b, c, d] = bytes else {
      return Err(crate::Error::http2_go_away(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidResetStreamFrameZeroId,
      ));
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*a, *b, *c, *d]).try_into()?,
      stream_id: fi.stream_id,
    })
  }

  pub(crate) fn bytes(&self) -> [u8; 13] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(4, 0, self.stream_id, FrameInitTy::Reset).bytes();
    let [j, k, l, m] = u32::from(self.error_code).to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m]
  }

  pub(crate) fn error_code(&self) -> Http2ErrorCode {
    self.error_code
  }
}
