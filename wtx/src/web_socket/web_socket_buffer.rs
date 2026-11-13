use crate::{
  collections::{VectorU8, VectorUsize},
  net::BufStreamReader,
  web_socket::read_frame_info::ReadFrameInfo,
};
use core::range::Range;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct WebSocketBuffer {
  pub(crate) network_buffer: BufStreamReader,
  pub(crate) reader_batches: VectorU8<(ReadFrameInfo, Range<usize>)>,
  // Used for decompression
  pub(crate) reader_buffer: VectorUsize<u8>,
  pub(crate) writer_buffer: VectorUsize<u8>,
}

impl WebSocketBuffer {
  /// New empty instance
  #[inline]
  pub const fn new() -> Self {
    Self {
      network_buffer: BufStreamReader::new(),
      reader_batches: VectorU8::new(),
      reader_buffer: VectorUsize::new(),
      writer_buffer: VectorUsize::new(),
    }
  }

  /// Pre-allocates the internal byte buffers.
  #[inline]
  pub fn with_capacity(network: usize, reader: usize, writer: usize) -> crate::Result<Self> {
    Ok(Self {
      network_buffer: BufStreamReader::with_capacity(network)?,
      reader_batches: VectorU8::new(),
      reader_buffer: VectorUsize::with_capacity(reader)?,
      writer_buffer: VectorUsize::with_capacity(writer)?,
    })
  }

  pub(crate) fn clear(&mut self) {
    let Self { network_buffer, reader_batches, reader_buffer, writer_buffer } = self;
    network_buffer.clear();
    reader_batches.clear();
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
