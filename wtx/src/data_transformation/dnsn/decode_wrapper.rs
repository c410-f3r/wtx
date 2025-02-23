use crate::misc::Lease;

/// Struct used for decoding elements in MySQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de> {
  pub(crate) bytes: &'de [u8],
}

impl<'de> DecodeWrapper<'de> {
  #[inline]
  pub(crate) fn _new(bytes: &'de [u8]) -> Self {
    Self { bytes }
  }
}

impl Lease<[u8]> for DecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
