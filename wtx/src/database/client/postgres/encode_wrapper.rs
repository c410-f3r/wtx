use crate::misc::{Lease, SuffixWriterFbvm};

/// Struct used for encoding elements in PostgreSQL.
#[derive(Debug)]
pub struct EncodeWrapper<'buffer, 'tmp> {
  buffer: &'tmp mut SuffixWriterFbvm<'buffer>,
}

impl<'buffer, 'tmp> EncodeWrapper<'buffer, 'tmp> {
  #[inline]
  pub(crate) fn new(buffer: &'tmp mut SuffixWriterFbvm<'buffer>) -> Self {
    Self { buffer }
  }

  /// Buffer used to encode messages that will be sent to PostgreSQL.
  #[inline]
  pub fn buffer(&mut self) -> &mut SuffixWriterFbvm<'buffer> {
    self.buffer
  }
}

impl Lease<[u8]> for EncodeWrapper<'_, '_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer._curr_bytes()
  }
}
