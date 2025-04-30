use crate::{collection::Vector, misc::Lease};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any> {
  pub(crate) vector: &'any mut Vector<u8>,
}

impl<'any> EncodeWrapper<'any> {
  /// New instance
  #[inline]
  pub const fn new(vector: &'any mut Vector<u8>) -> Self {
    Self { vector }
  }
}

impl Lease<[u8]> for EncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.vector
  }
}
