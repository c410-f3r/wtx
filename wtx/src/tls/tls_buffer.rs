use crate::misc::{Lease, LeaseMut, net::PartitionedFilledBuffer};

#[derive(Debug)]
pub struct TlsBuffer {
  pub(crate) network_buffer: PartitionedFilledBuffer,
}

impl Lease<TlsBuffer> for TlsBuffer {
  #[inline]
  fn lease(&self) -> &Self {
    self
  }
}

impl LeaseMut<TlsBuffer> for TlsBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut Self {
    self
  }
}

impl Default for TlsBuffer {
  #[inline]
  fn default() -> Self {
    Self { network_buffer: PartitionedFilledBuffer::default() }
  }
}
