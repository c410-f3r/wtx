use crate::misc::Lease;

/// Struct used to represent decoded columns in MySQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de> {
  bytes: &'de [u8],
  is_hello_retry_request: bool,
}

impl<'de> DecodeWrapper<'de> {
  pub(crate) const fn from_bytes(bytes: &'de [u8]) -> Self {
    Self { bytes, is_hello_retry_request: false }
  }

  pub(crate) const fn new(bytes: &'de [u8], is_hello_retry_request: bool) -> Self {
    Self { bytes, is_hello_retry_request }
  }

  #[inline]
  pub(crate) const fn bytes(&self) -> &'de [u8] {
    self.bytes
  }

  #[inline]
  pub(crate) const fn bytes_mut(&mut self) -> &mut &'de [u8] {
    &mut self.bytes
  }

  #[inline]
  pub const fn is_hello_retry_request(&self) -> bool {
    self.is_hello_retry_request
  }
}

impl Lease<[u8]> for DecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
