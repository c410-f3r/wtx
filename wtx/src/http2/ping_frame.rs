use crate::{
  http::u31::U31,
  http2::{
    Http2Error, Http2ErrorCode,
    common_flags::CommonFlags,
    frame_init::{FrameInit, FrameInitTy},
  },
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PingFrame {
  cf: CommonFlags,
  payload: [u8; 8],
}

impl PingFrame {
  pub(crate) const fn new(cf: CommonFlags, payload: [u8; 8]) -> Self {
    Self { cf, payload }
  }

  pub(crate) const fn read(bytes: &[u8], mut fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidPingFrameNonZeroId,
      ));
    }
    fi.cf.only_ack();
    let [b0, b1, b2, b3, b4, b5, b6, b7] = bytes else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidPingFrameBytes,
      ));
    };
    Ok(Self::new(fi.cf, [*b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7]))
  }

  pub(crate) const fn bytes(&self) -> [u8; 17] {
    let fi = FrameInit::new(self.cf, 8, U31::ZERO, FrameInitTy::Ping);
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8] = fi.bytes();
    let [b9, b10, b11, b12, b13, b14, b15, b16] = self.payload;
    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16]
  }

  pub(crate) const fn has_ack(&self) -> bool {
    self.cf.has_ack()
  }

  pub(crate) const fn set_ack(&mut self) {
    self.cf.set_ack();
  }
}
