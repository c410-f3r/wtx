use crate::http2::{
  common_flags::CommonFlags,
  frame_init::{FrameInit, FrameInitTy},
  u31::U31,
};

#[derive(Debug)]
pub(crate) struct ContinuationFrame {
  cf: CommonFlags,
  stream_id: U31,
}

impl ContinuationFrame {
  pub(crate) const fn new(stream_id: U31) -> Self {
    Self { cf: CommonFlags::empty(), stream_id }
  }

  pub(crate) const fn bytes(&self) -> [u8; 9] {
    FrameInit::new(self.cf, 0, self.stream_id, FrameInitTy::Continuation).bytes()
  }

  pub(crate) const fn set_eoh(&mut self) {
    self.cf.set_eoh();
  }
}
