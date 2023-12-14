use crate::{misc::PartitionedFilledBuffer, web_socket::misc::FilledBuffer};

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
  pub fn with_capacity(decompression_buffer_cap: usize, network_buffer_cap: usize) -> Self {
    Self {
      db: FilledBuffer::with_capacity(decompression_buffer_cap),
      nb: PartitionedFilledBuffer::with_capacity(network_buffer_cap),
    }
  }

  pub(crate) fn parts_mut(&mut self) -> (&mut FilledBuffer, &mut PartitionedFilledBuffer) {
    (&mut self.db, &mut self.nb)
  }
}
