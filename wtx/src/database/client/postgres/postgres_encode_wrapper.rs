use crate::misc::{Lease, SuffixWriterFbvm};

/// Struct used for encoding elements in PostgreSQL.
#[derive(Debug)]
pub struct PostgresEncodeWrapper<'inner, 'outer> {
  buffer: &'outer mut SuffixWriterFbvm<'inner>,
}

impl<'inner, 'outer> PostgresEncodeWrapper<'inner, 'outer> {
  pub(crate) const fn new(buffer: &'outer mut SuffixWriterFbvm<'inner>) -> Self {
    Self { buffer }
  }

  /// Buffer used to encode messages that will be sent to PostgreSQL.
  pub const fn buffer(&mut self) -> &mut SuffixWriterFbvm<'inner> {
    self.buffer
  }
}

impl Lease<[u8]> for PostgresEncodeWrapper<'_, '_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer.curr_bytes()
  }
}
