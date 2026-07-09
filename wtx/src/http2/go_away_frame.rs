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
pub(crate) struct GoAwayFrame {
  error_code: Http2ErrorCode,
  last_stream_id: U31,
}

impl GoAwayFrame {
  pub(crate) const fn new(error_code: Http2ErrorCode, last_stream_id: U31) -> Self {
    Self { error_code, last_stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 17] {
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8] =
      FrameInit::new(CommonFlags::empty(), 8, U31::ZERO, FrameInitTy::GoAway).bytes();
    let [b9, b10, b11, b12] = self.last_stream_id.to_be_bytes();
    let [b13, b14, b15, b16] = u32::from(self.error_code).to_be_bytes();
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16]
  }

  pub(crate) const fn error_code(&self) -> Http2ErrorCode {
    self.error_code
  }

  pub(crate) fn read(data: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(protocol_err(Http2Error::InvalidGoAwayFrameNonZeroId));
    }
    let [b0, b1, b2, b3, b4, b5, b6, b7, ..] = data else {
      return Err(protocol_err(Http2Error::InvalidGoAwayFrameBytes));
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*b4, *b5, *b6, *b7]).try_into()?,
      last_stream_id: U31::from_u32(u32::from_be_bytes([*b0, *b1, *b2, *b3])),
    })
  }
}
