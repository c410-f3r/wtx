use crate::misc::{Lease, LeaseMut};

#[derive(Debug)]
pub struct ExecutorBuffer {}

impl Lease<ExecutorBuffer> for ExecutorBuffer {
  #[inline]
  fn lease(&self) -> &ExecutorBuffer {
    self
  }
}

impl LeaseMut<ExecutorBuffer> for ExecutorBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut ExecutorBuffer {
    self
  }
}
