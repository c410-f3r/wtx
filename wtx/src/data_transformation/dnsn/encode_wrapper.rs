use crate::misc::{Lease, Vector};

/// Struct used for encoding elements in MySQL.
#[derive(Debug)]
pub struct EncodeWrapper<'any, DRSR> {
  pub(crate) drsr: &'any mut DRSR,
  pub(crate) vector: &'any mut Vector<u8>,
}

impl<'any, DRSR> EncodeWrapper<'any, DRSR> {
  #[inline]
  pub(crate) fn new(drsr: &'any mut DRSR, vector: &'any mut Vector<u8>) -> Self {
    Self { drsr, vector }
  }
}

impl<'any, DRSR> Lease<[u8]> for EncodeWrapper<'any, DRSR> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.vector
  }
}
