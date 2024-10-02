use crate::misc::{FilledBuffer, Lease, LeaseMut, PartitionedFilledBuffer, VectorError};

#[derive(Debug, Default)]
#[doc = _internal_buffer_doc!()]
pub struct WebSocketBuffer {
  /// Decompression buffer
  pub(crate) db: FilledBuffer,
  /// Network buffer
  pub(crate) nb: PartitionedFilledBuffer,
}

impl WebSocketBuffer {
  /// The elements used internally will be able to hold at least the specified amounts.
  #[inline]
  pub fn with_capacity(
    decompression_buffer_cap: usize,
    network_buffer_cap: usize,
  ) -> Result<Self, VectorError> {
    Ok(Self {
      db: FilledBuffer::_with_capacity(decompression_buffer_cap)?,
      nb: PartitionedFilledBuffer::_with_capacity(network_buffer_cap)?,
    })
  }

  pub(crate) fn _clear(&mut self) {
    let Self { db, nb } = self;
    db._clear();
    nb._clear();
  }

  pub(crate) fn _parts_mut(&mut self) -> (&mut FilledBuffer, &mut PartitionedFilledBuffer) {
    (&mut self.db, &mut self.nb)
  }
}

impl Lease<WebSocketBuffer> for WebSocketBuffer {
  #[inline]
  fn lease(&self) -> &WebSocketBuffer {
    self
  }
}

impl LeaseMut<WebSocketBuffer> for WebSocketBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut WebSocketBuffer {
    self
  }
}
