use crate::{misc::Lease, tls::CipherSuite};

/// Struct used to represent decoded elements.
#[derive(Debug, PartialEq)]
pub(crate) struct TlsDecodeWrapper<'de> {
  bytes: &'de [u8],
  cipher_suite: CipherSuite,
  is_hello_retry_request: bool,
  is_x509: bool,
}

impl<'de> TlsDecodeWrapper<'de> {
  pub(crate) fn from_bytes(bytes: &'de [u8]) -> Self {
    Self {
      bytes,
      cipher_suite: CipherSuite::default(),
      is_hello_retry_request: false,
      is_x509: false,
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
  pub(crate) const fn cipher_suite(&self) -> CipherSuite {
    self.cipher_suite
  }

  #[inline]
  pub(crate) const fn cipher_suite_mut(&mut self) -> &mut CipherSuite {
    &mut self.cipher_suite
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request(&self) -> bool {
    self.is_hello_retry_request
  }

  #[inline]
  pub(crate) const fn is_hello_retry_request_mut(&mut self) -> &mut bool {
    &mut self.is_hello_retry_request
  }
}

impl Lease<[u8]> for TlsDecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}
