use crate::misc::{Lease, Vector};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any> {
  pub(crate) vector: &'any mut Vector<u8>,
}

impl<'any> EncodeWrapper<'any> {
  #[inline]
  pub(crate) fn _new(vector: &'any mut Vector<u8>) -> Self {
    Self { vector }
  }
}

impl<'any> Lease<[u8]> for EncodeWrapper<'any> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.vector
  }
}
