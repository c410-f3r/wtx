use crate::misc::{Lease, SuffixWriterMut};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any> {
  buffer: SuffixWriterMut<'any>,
}

impl<'any> EncodeWrapper<'any> {
  pub(crate) const fn new(buffer: SuffixWriterMut<'any>) -> Self {
    Self { buffer }
  }

  #[inline]
  pub(crate) const fn buffer(&mut self) -> &mut SuffixWriterMut<'any> {
    &mut self.buffer
  }
}

impl Lease<[u8]> for EncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer.curr_bytes()
  }
}
