use crate::{
  collection::{ArrayVector, ArrayVectorU8},
  misc::{Lease, LeaseMut},
  tls::{
    CipherSuiteTy, MAX_KEY_SHARES_LEN, MaxFragmentLength, NamedGroup,
    protocol::{
      key_share_entry::KeyShareEntry, offered_psks::OfferedPsks, server_name_list::ServerNameList,
      signature_scheme::SignatureScheme,
    },
  },
};

#[derive(Debug)]
pub struct TlsConfig<'any> {
  pub(crate) inner: TlsConfigInner<'any>,
}

impl<'any> TlsConfig<'any> {
  /// Certificate Authority
  #[inline]
  pub fn set_ca(&mut self, elem: &'any [u8]) -> &mut Self {
    self.inner.root_ca = Some(elem);
    self
  }
}

impl Default for TlsConfig<'_> {
  #[inline]
  fn default() -> Self {
    Self {
      inner: TlsConfigInner {
        root_ca: None,
        certificate: None,
        cipher_suites: ArrayVector::from_array(CipherSuiteTy::all()),
        key_shares: ArrayVector::from_array([
          KeyShareEntry { group: NamedGroup::X25519, opaque: &[] },
          KeyShareEntry { group: NamedGroup::Secp256r1, opaque: &[] },
        ]),
        max_fragment_length: None,
        named_groups: ArrayVector::from_array(NamedGroup::all()),
        offered_psks: OfferedPsks { offered_psks: ArrayVectorU8::new() },
        secret_key: &[],
        server_name: None,
        signature_algorithms: ArrayVector::from_array(SignatureScheme::PRIORITY),
        signature_algorithms_cert: ArrayVector::from_array(SignatureScheme::PRIORITY),
      },
    }
  }
}

#[derive(Debug)]
pub(crate) struct TlsConfigInner<'any> {
  pub(crate) root_ca: Option<&'any [u8]>,
  pub(crate) certificate: Option<&'any [u8]>,
  pub(crate) secret_key: &'any [u8],

  pub(crate) cipher_suites: ArrayVectorU8<CipherSuiteTy, { CipherSuiteTy::len() }>,
  pub(crate) key_shares: ArrayVectorU8<KeyShareEntry<'any>, MAX_KEY_SHARES_LEN>,
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) named_groups: ArrayVectorU8<NamedGroup, { NamedGroup::len() }>,
  pub(crate) offered_psks: OfferedPsks<'any>,
  pub(crate) server_name: Option<ServerNameList<'any>>,
  pub(crate) signature_algorithms: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
  pub(crate) signature_algorithms_cert: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
}

impl<'any> Lease<TlsConfigInner<'any>> for TlsConfigInner<'any> {
  #[inline]
  fn lease(&self) -> &TlsConfigInner<'any> {
    self
  }
}

impl<'any> LeaseMut<TlsConfigInner<'any>> for TlsConfigInner<'any> {
  #[inline]
  fn lease_mut(&mut self) -> &mut TlsConfigInner<'any> {
    self
  }
}
