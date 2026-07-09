use crate::{
  http::U31,
  http2::{
    Http2Error, Http2ErrorCode,
    common_flags::CommonFlags,
    frame_init::{FrameInit, FrameInitTy},
    misc::protocol_err,
  },
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ResetStreamFrame {
  error_code: Http2ErrorCode,
  stream_id: U31,
}

impl ResetStreamFrame {
  pub(crate) const fn new(error_code: Http2ErrorCode, stream_id: U31) -> Self {
    Self { error_code, stream_id }
  }

  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_zero() {
      return Err(protocol_err(Http2Error::InvalidResetStreamFrameBytes));
    }
    let [b0, b1, b2, b3] = bytes else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidResetStreamFrameZeroId,
      ));
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*b0, *b1, *b2, *b3])
        .try_into()
        .unwrap_or(Http2ErrorCode::InternalError),
      stream_id: fi.stream_id,
    })
  }

  pub(crate) fn bytes(&self) -> [u8; 13] {
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8] =
      FrameInit::new(CommonFlags::empty(), 4, self.stream_id, FrameInitTy::Reset).bytes();
    let [b9, b10, b11, b12] = u32::from(self.error_code).to_be_bytes();
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12]
  }

  pub(crate) const fn error_code(&self) -> Http2ErrorCode {
    self.error_code
  }
}
