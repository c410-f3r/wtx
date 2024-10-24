use crate::http2::{
  common_flags::CommonFlags,
  frame_init::{FrameInit, FrameInitTy},
  misc::protocol_err,
  u31::U31,
  Http2Error, Http2ErrorCode,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ResetStreamFrame {
  error_code: Http2ErrorCode,
  stream_id: U31,
}

impl ResetStreamFrame {
  #[inline]
  pub(crate) const fn new(error_code: Http2ErrorCode, stream_id: U31) -> Self {
    Self { error_code, stream_id }
  }

  #[inline]
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return Err(protocol_err(Http2Error::InvalidResetStreamFrameBytes));
    }
    let [a, b, c, d] = bytes else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Some(Http2Error::InvalidResetStreamFrameZeroId),
      ));
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*a, *b, *c, *d])
        .try_into()
        .unwrap_or(Http2ErrorCode::InternalError),
      stream_id: fi.stream_id,
    })
  }

  #[inline]
  pub(crate) fn bytes(&self) -> [u8; 13] {
    let [a, b, c, d, e, f, g, h, i] =
      FrameInit::new(CommonFlags::empty(), 4, self.stream_id, FrameInitTy::Reset).bytes();
    let [j, k, l, m] = u32::from(self.error_code).to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m]
  }

  #[inline]
  pub(crate) const fn error_code(&self) -> Http2ErrorCode {
    self.error_code
  }
}
