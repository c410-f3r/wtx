use crate::{
  collections::VectorUsize,
  misc::{Lease, LeaseMut},
};

/// Struct used to represent decoded elements.
#[derive(Debug, PartialEq)]
pub(crate) struct _SmtpDecodeWrapper<'de> {
  bytes: &'de [u8],
}

impl<'de> _SmtpDecodeWrapper<'de> {
  pub(crate) fn _from_bytes(bytes: &'de [u8]) -> Self {
    Self { bytes }
  }
}

impl Lease<[u8]> for _SmtpDecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

/// Struct used for encoding TLS elements.
#[derive(Debug)]
pub(crate) struct _SmtpEncodeWrapper<'any> {
  buffer: &'any mut VectorUsize<u8>,
}

impl<'any> _SmtpEncodeWrapper<'any> {
  pub(crate) const fn _from_buffer(buffer: &'any mut VectorUsize<u8>) -> Self {
    Self { buffer }
  }

  #[inline]
  pub(crate) const fn _buffer(&mut self) -> &mut VectorUsize<u8> {
    self.buffer
  }
}

impl Lease<[u8]> for _SmtpEncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer
  }
}

impl LeaseMut<[u8]> for _SmtpEncodeWrapper<'_> {
  #[inline]
  fn lease_mut(&mut self) -> &mut [u8] {
    self.buffer
  }
}
