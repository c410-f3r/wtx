use crate::{collection::Vector, misc::Lease};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any> {
  buffer: &'any mut Vector<u8>,
}

impl<'any> EncodeWrapper<'any> {
  #[inline]
  pub(crate) fn new(buffer: &'any mut Vector<u8>) -> Self {
    Self { buffer }
  }

  /// Buffer used to encode messages that will be sent to MySQL.
  #[inline]
  pub fn buffer(&mut self) -> &mut Vector<u8> {
    self.buffer
  }
}

impl Lease<[u8]> for EncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer
  }
}
