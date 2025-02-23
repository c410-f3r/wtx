use crate::misc::{Lease, Vector};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any> {
  sw: &'any mut Vector<u8>,
}

impl<'any> EncodeWrapper<'any> {
  #[inline]
  pub(crate) fn new(sw: &'any mut Vector<u8>) -> Self {
    Self { sw }
  }

  /// See [`FilledBufferWriter`].
  #[inline]
  pub fn sw(&mut self) -> &mut Vector<u8> {
    self.sw
  }
}

impl Lease<[u8]> for EncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.sw
  }
}
