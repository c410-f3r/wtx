use crate::misc::{Lease, LeaseMut, partitioned_filled_buffer::PartitionedFilledBuffer};

#[derive(Debug)]
pub struct TlsStreamBuffer {
  pub(crate) network_buffer: PartitionedFilledBuffer,
}

impl Lease<TlsStreamBuffer> for TlsStreamBuffer {
  #[inline]
  fn lease(&self) -> &Self {
    self
  }
}

impl LeaseMut<TlsStreamBuffer> for TlsStreamBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut Self {
    self
  }
}

impl Default for TlsStreamBuffer {
  #[inline]
  fn default() -> Self {
    Self { network_buffer: PartitionedFilledBuffer::default() }
  }
}
