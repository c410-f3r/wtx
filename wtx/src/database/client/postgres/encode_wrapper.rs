use crate::misc::{Lease, SuffixWriterFbvm};

/// Struct used for encoding elements in PostgreSQL.
#[derive(Debug)]
pub struct EncodeWrapper<'buffer, 'tmp> {
  sw: &'tmp mut SuffixWriterFbvm<'buffer>,
}

impl<'buffer, 'tmp> EncodeWrapper<'buffer, 'tmp> {
  #[inline]
  pub(crate) fn new(sw: &'tmp mut SuffixWriterFbvm<'buffer>) -> Self {
    Self { sw }
  }

  /// See [`FilledBufferWriter`].
  #[inline]
  pub fn sw(&mut self) -> &mut SuffixWriterFbvm<'buffer> {
    self.sw
  }
}

impl Lease<[u8]> for EncodeWrapper<'_, '_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.sw._curr_bytes()
  }
}
