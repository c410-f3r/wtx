use crate::misc::Lease;

/// Struct used for decoding elements in MySQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'any, 'de, DRSR> {
  pub(crate) bytes: &'de [u8],
  pub(crate) drsr: &'any mut DRSR,
}

impl<'any, 'de, DRSR> DecodeWrapper<'any, 'de, DRSR> {
  #[inline]
  pub(crate) fn new(bytes: &'de [u8], drsr: &'any mut DRSR) -> Self {
    Self { bytes, drsr }
  }
}

impl<DRSR> Lease<[u8]> for DecodeWrapper<'_, '_, DRSR> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
