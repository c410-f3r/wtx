use crate::{misc::Lease, tls::protocol::signature_scheme::SignatureScheme};

/// Struct used to represent decoded columns in MySQL.
#[derive(Debug, PartialEq)]
pub struct DecodeWrapper<'de> {
  bytes: &'de [u8],
  is_hello_retry_request: bool,
  is_x509: bool,
  signature_scheme: SignatureScheme,
}

impl<'de> DecodeWrapper<'de> {
  pub(crate) const fn from_bytes(bytes: &'de [u8]) -> Self {
    Self {
      bytes,
      is_hello_retry_request: false,
      is_x509: false,
      signature_scheme: SignatureScheme::Ed25519,
    }
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
  pub(crate) const fn is_hello_retry_request(&self) -> bool {
    self.is_hello_retry_request
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request_mut(&mut self) -> &mut bool {
    &mut self.is_hello_retry_request
  }

  #[inline]
  pub(crate) const fn signature_scheme(&self) -> SignatureScheme {
    self.signature_scheme
  }

  #[inline]
  pub(crate) const fn signature_scheme_mut(&mut self) -> &mut SignatureScheme {
    &mut self.signature_scheme
  }
}

impl Lease<[u8]> for DecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
