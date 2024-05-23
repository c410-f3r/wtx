use crate::http2::{FrameInit, FrameInitTy, Http2Error, Http2ErrorCode, U31};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct GoAwayFrame {
  error_code: Http2ErrorCode,
  last_stream_id: U31,
}

impl GoAwayFrame {
  #[inline]
  pub(crate) fn new(error_code: Http2ErrorCode, last_stream_id: U31) -> Self {
    Self { error_code, last_stream_id }
  }

  #[inline]
  pub(crate) fn bytes(&self) -> [u8; 17] {
    let [a, b, c, d, e, f, g, h, i] = FrameInit::new(8, 0, U31::ZERO, FrameInitTy::GoAway).bytes();
    let [j, k, l, m] = self.last_stream_id.to_be_bytes();
    let [n, o, p, q] = u32::from(self.error_code).to_be_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q]
  }

  #[inline]
  pub(crate) fn error_code(&self) -> Http2ErrorCode {
    self.error_code
  }

  #[inline]
  pub(crate) fn read(data: &[u8], fi: FrameInit) -> crate::Result<Self> {
    if fi.stream_id.is_not_zero() {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidGoAwayFrameNonZeroId));
    }
    let [a, b, c, d, e, f, g, h] = data else {
      return Err(crate::Error::http2_go_away_generic(Http2Error::InvalidGoAwayFrameBytes));
    };
    Ok(Self {
      error_code: u32::from_be_bytes([*e, *f, *g, *h]).try_into()?,
      last_stream_id: U31::from_u32(u32::from_be_bytes([*a, *b, *c, *d])),
    })
  }
}
