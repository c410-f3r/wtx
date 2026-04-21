use crate::misc::{Lease, LeaseMut, SuffixWriterMut};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any> {
  buffer: SuffixWriterMut<'any>,
  is_hello_retry_request: bool,
}

impl<'any> EncodeWrapper<'any> {
  pub(crate) const fn from_buffer(buffer: SuffixWriterMut<'any>) -> Self {
    Self { buffer, is_hello_retry_request: false }
  }

  #[inline]
  pub(crate) const fn buffer(&mut self) -> &mut SuffixWriterMut<'any> {
    &mut self.buffer
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request(&self) -> bool {
    self.is_hello_retry_request
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request_mut(&mut self) -> &mut bool {
    &mut self.is_hello_retry_request
  }
}

impl Lease<[u8]> for EncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer.curr_bytes()
  }
}

impl<'any> Lease<SuffixWriterMut<'any>> for EncodeWrapper<'any> {
  #[inline]
  fn lease(&self) -> &SuffixWriterMut<'any> {
    &self.buffer
  }
}

impl<'any> LeaseMut<SuffixWriterMut<'any>> for EncodeWrapper<'any> {
  #[inline]
  fn lease_mut(&mut self) -> &mut SuffixWriterMut<'any> {
    &mut self.buffer
  }
}
