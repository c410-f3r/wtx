use crate::{
  collection::Vector,
  misc::{Lease, LeaseMut, net::PartitionedFilledBuffer},
};

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct WebSocketBuffer {
  pub(crate) network_buffer: PartitionedFilledBuffer,
  // Used for decompression
  pub(crate) reader_buffer: Vector<u8>,
  pub(crate) writer_buffer: Vector<u8>,
}

impl WebSocketBuffer {
  /// New empty instance
  #[inline]
  pub fn new() -> Self {
    Self {
      network_buffer: PartitionedFilledBuffer::default(),
      reader_buffer: Vector::new(),
      writer_buffer: Vector::new(),
    }
  }

  /// The elements used internally will be able to hold at least the specified amounts.
  #[inline]
  pub fn with_capacity(
    network_buffer_cap: usize,
    reader_buffer_cap: usize,
    writer_buffer_cap: usize,
  ) -> crate::Result<Self> {
    Ok(Self {
      network_buffer: PartitionedFilledBuffer::with_capacity(network_buffer_cap)?,
      reader_buffer: Vector::with_capacity(reader_buffer_cap)?,
      writer_buffer: Vector::with_capacity(writer_buffer_cap)?,
    })
  }

  #[cfg(feature = "web-socket-handshake")]
  pub(crate) fn clear(&mut self) {
    let Self { network_buffer, reader_buffer, writer_buffer } = self;
    network_buffer.clear();
    reader_buffer.clear();
    writer_buffer.clear();
  }
}

impl Default for WebSocketBuffer {
  #[inline]
  fn default() -> Self {
    Self::new()
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
