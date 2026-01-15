use crate::{
  collection::Vector,
  misc::{Lease, LeaseMut, net::PartitionedFilledBuffer},
};

#[derive(Debug)]
pub struct TlsBuffer {
  pub(crate) network_buffer: PartitionedFilledBuffer,
  pub(crate) write_buffer: Vector<u8>,
}

impl TlsBuffer {
  /// Clears internal state
  pub fn clear(&mut self) {
    let Self { network_buffer, write_buffer } = self;
    network_buffer.clear();
    write_buffer.clear();
  }
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
    Self { network_buffer: PartitionedFilledBuffer::default(), write_buffer: Vector::new() }
  }
}
