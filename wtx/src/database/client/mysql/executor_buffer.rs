use crate::misc::{Lease, LeaseMut, partitioned_filled_buffer::PartitionedFilledBuffer};

#[derive(Debug)]
pub struct ExecutorBuffer {
  pub(crate) nb: PartitionedFilledBuffer,
}

impl ExecutorBuffer {
  /// New instance
  #[inline]
  pub fn new() -> Self {
    Self { nb: PartitionedFilledBuffer::new() }
  }
}

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
