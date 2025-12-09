use crate::tls::{MaxFragmentLength, NamedGroup, SignatureScheme, cipher_suite::CipherSuiteTy};

#[derive(Debug)]
pub struct TlsConfig<'any> {
  pub(crate) ca: Option<&'any [u8]>,
  pub(crate) cert: Option<&'any [u8]>,
  pub(crate) cipher_suites: &'any [CipherSuiteTy],
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) named_group_list: &'any [NamedGroup],
  pub(crate) psk: Option<(&'any [u8], &'any [&'any [u8]])>,
  pub(crate) secret_key: &'any [u8],
  pub(crate) server_name: Option<&'any str>,
  pub(crate) signature_schemes: &'any [SignatureScheme],
}

impl<'any> TlsConfig<'any> {
  /// Certificate Authority
  #[inline]
  pub fn set_ca(&mut self, elem: &'any [u8]) -> &mut Self {
    self.ca = Some(elem);
    self
  }
}

impl Default for TlsConfig<'_> {
  #[inline]
  fn default() -> Self {
    Self {
      ca: None,
      cert: None,
      cipher_suites: &[],
      max_fragment_length: None,
      named_group_list: &[],
      psk: None,
      secret_key: &[],
      server_name: None,
      signature_schemes: &[],
    }
  }
}
