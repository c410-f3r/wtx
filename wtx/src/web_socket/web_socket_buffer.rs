use crate::misc::{FilledBuffer, Lease, LeaseMut, PartitionedFilledBuffer, VectorError};

#[derive(Debug, Default)]
#[doc = _internal_buffer_doc!()]
pub struct WebSocketBuffer {
  pub(crate) network_buffer: PartitionedFilledBuffer,
  pub(crate) writer_buffer: FilledBuffer,
  pub(crate) reader_buffer_first: FilledBuffer,
  pub(crate) reader_buffer_second: FilledBuffer,
}

impl WebSocketBuffer {
  /// New empty instance
  #[inline]
  pub fn new() -> Self {
    Self {
      network_buffer: PartitionedFilledBuffer::new(),
      reader_buffer_first: FilledBuffer::_new(),
      reader_buffer_second: FilledBuffer::_new(),
      writer_buffer: FilledBuffer::_new(),
    }
  }

  /// The elements used internally will be able to hold at least the specified amounts.
  #[inline]
  pub fn with_capacity(
    network_buffer_cap: usize,
    reader_buffer_cap: usize,
    writer_buffer_cap: usize,
  ) -> Result<Self, VectorError> {
    Ok(Self {
      network_buffer: PartitionedFilledBuffer::_with_capacity(network_buffer_cap)?,
      reader_buffer_first: FilledBuffer::_with_capacity(reader_buffer_cap)?,
      reader_buffer_second: FilledBuffer::_with_capacity(reader_buffer_cap)?,
      writer_buffer: FilledBuffer::_with_capacity(writer_buffer_cap)?,
    })
  }

  pub(crate) fn _clear(&mut self) {
    let Self { network_buffer, reader_buffer_first, reader_buffer_second, writer_buffer } = self;
    network_buffer._clear();
    reader_buffer_first._clear();
    reader_buffer_second._clear();
    writer_buffer._clear();
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
