use crate::http2::{FrameHeaderTy, FrameInit, ACK_MASK, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PingFrame {
  flags: u8,
  payload: [u8; 8],
}

impl PingFrame {
  #[inline]
  pub(crate) fn read(bytes: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(crate::http2::ErrorCode::ProtocolError.into());
    }
    let [a, b, c, d, e, f, g, h] = bytes else {
      return Err(crate::http2::ErrorCode::FrameSizeError.into());
    };
    Ok(Self { flags: fi.flags & ACK_MASK, payload: [*a, *b, *c, *d, *e, *f, *g, *h] })
  }

  #[inline]
  pub(crate) fn bytes(&self) -> [u8; 17] {
    let fi = FrameInit::new(8, self.flags, U31::ZERO, FrameHeaderTy::Ping);
    let [a, b, c, d, e, f, g, h, i] = fi.bytes();
    let [j, k, l, m, n, o, p, q] = self.payload;
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q]
  }

  #[inline]
  pub(crate) fn is_ack(&self) -> bool {
    self.flags == ACK_MASK
  }

  #[inline]
  pub(crate) fn set_ack(&mut self) {
    self.flags = ACK_MASK;
  }
}
