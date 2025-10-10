use crate::misc::Lease;

/// Struct used for decoding different formats.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de> {
  pub(crate) bytes: &'de [u8],
}

impl<'de> DecodeWrapper<'de> {
  /// New instance
  #[inline]
  pub const fn new(bytes: &'de [u8]) -> Self {
    Self { bytes }
  }
}

impl Lease<[u8]> for DecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
