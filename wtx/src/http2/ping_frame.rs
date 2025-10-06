use crate::http2::{
  Http2Error, Http2ErrorCode,
  common_flags::CommonFlags,
  frame_init::{FrameInit, FrameInitTy},
  u31::U31,
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
    let [a, b, c, d, e, f, g, h] = bytes else {
      return Err(crate::Error::Http2ErrorGoAway(
        Http2ErrorCode::FrameSizeError,
        Http2Error::InvalidPingFrameBytes,
      ));
    };
    Ok(Self::new(fi.cf, [*a, *b, *c, *d, *e, *f, *g, *h]))
  }

  pub(crate) const fn bytes(&self) -> [u8; 17] {
    let fi = FrameInit::new(self.cf, 8, U31::ZERO, FrameInitTy::Ping);
    let [a, b, c, d, e, f, g, h, i] = fi.bytes();
    let [j, k, l, m, n, o, p, q] = self.payload;
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q]
  }

  pub(crate) const fn has_ack(&self) -> bool {
    self.cf.has_ack()
  }

  pub(crate) const fn set_ack(&mut self) {
    self.cf.set_ack();
  }
}
