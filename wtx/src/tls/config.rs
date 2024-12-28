use crate::tls::{cipher_suite::CipherSuite, MaxFragmentLength, NamedGroup, SignatureScheme};

#[derive(Debug)]
pub struct Config<'any> {
  pub(crate) ca: Option<&'any [u8]>,
  pub(crate) cert: Option<&'any [u8]>,
  pub(crate) cipher_suites: &'any [CipherSuite],
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) named_group_list: &'any [NamedGroup],
  pub(crate) priv_key: &'any [u8],
  pub(crate) psk: Option<(&'any [u8], &'any [&'any [u8]])>,
  pub(crate) server_name: Option<&'any str>,
  pub(crate) signature_schemes: &'any [SignatureScheme],
}

impl<'any> Config<'any> {
  #[inline]
  pub fn new() -> Self {
    Self {
      ca: None,
      cert: None,
      cipher_suites: &[],
      max_fragment_length: None,
      named_group_list: &[],
      priv_key: &[],
      psk: None,
      server_name: None,
      signature_schemes: &[],
    }
  }

  /// Certificate Authority
  #[inline]
  pub fn set_ca(&mut self, elem: &'any [u8]) -> &mut Self {
    self.ca = Some(elem);
    self
  }
}
