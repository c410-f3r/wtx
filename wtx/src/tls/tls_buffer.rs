use crate::{collections::Vector, net::BufStreamReader};

#[derive(Debug, Default)]
#[doc = _internal_buffer_doc!()]
pub struct TlsBuffer {
  pub(crate) reader_buffer: BufStreamReader,
  pub(crate) writer_buffer: Vector<u8>,
}

impl TlsBuffer {
  /// Empty instance
  #[inline]
  pub const fn new() -> Self {
    Self { reader_buffer: BufStreamReader::new(), writer_buffer: Vector::new() }
  }
}
