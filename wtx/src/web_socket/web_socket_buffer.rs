use crate::{collections::Vector, stream::BufStreamReader};

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct WebSocketBuffer {
  pub(crate) network_buffer: BufStreamReader,
  // Used for decompression
  pub(crate) reader_buffer: Vector<u8>,
  pub(crate) writer_buffer: Vector<u8>,
}

impl WebSocketBuffer {
  /// New empty instance
  #[inline]
  pub fn new() -> Self {
    Self {
      network_buffer: BufStreamReader::new(),
      reader_buffer: Vector::new(),
      writer_buffer: Vector::new(),
    }
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
