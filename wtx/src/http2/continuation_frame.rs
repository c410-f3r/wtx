use crate::http2::{FrameInit, FrameInitTy, EOH_MASK, U31};

#[derive(Debug)]
pub(crate) struct ContinuationFrame {
  flag: u8,
  stream_id: U31,
}

impl ContinuationFrame {
  pub(crate) fn new(stream_id: U31) -> Self {
    Self { flag: 0, stream_id }
  }

  pub(crate) fn bytes(&self) -> [u8; 9] {
    FrameInit::new(0, self.flag, self.stream_id, FrameInitTy::Continuation).bytes()
  }

  pub(crate) fn set_eoh(&mut self) {
    self.flag |= EOH_MASK;
  }
}
