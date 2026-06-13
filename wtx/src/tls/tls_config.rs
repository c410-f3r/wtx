use core::fmt::Debug;

use crate::{
  collection::{ArrayVector, ArrayVectorU8},
  misc::{Lease, LeaseMut},
  tls::{
    CipherSuite, MAX_ALPN_LEN, MAX_KEY_SHARES_LEN, MaxFragmentLength, NamedGroup,
    protocol::{
      alpn::Alpn, key_share_entry::KeyShareEntry, offered_psks::OfferedPsks,
      server_name_list::ServerNameList, signature_scheme::SignatureScheme,
    },
  },
  x509::{CvPolicy, CvTrustAnchor},
};

#[derive(Clone)]
pub struct TlsConfig<'any> {
  pub(crate) alpn: Alpn<'any>,
  pub(crate) cipher_suites: ArrayVectorU8<CipherSuite, { CipherSuite::len() }>,
  pub(crate) cv_policy: CvPolicy<'any, 'any>,
  pub(crate) key_shares: ArrayVectorU8<KeyShareEntry<'any>, MAX_KEY_SHARES_LEN>,
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) named_groups: ArrayVectorU8<NamedGroup, { NamedGroup::len() }>,
  pub(crate) offered_psks: OfferedPsks<'any>,
  pub(crate) public_key: &'any [u8],
  pub(crate) secret_key: &'any [u8],
  pub(crate) server_name: Option<ServerNameList<'any>>,
  pub(crate) signature_algorithms_cert: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
  pub(crate) signature_algorithms: ArrayVectorU8<SignatureScheme, { SignatureScheme::len() }>,
  pub(crate) trust_anchors: &'any [CvTrustAnchor<'any>],
}

impl<'any> TlsConfig<'any> {
  /// New instance that doesn't incorporate any initial certificate, which will likely make
  /// connections fail. However, it is still possible to add certificates using mutable methods.
  pub const fn uncertified() -> Self {
    Self::new()
  }

  /// New instance from the trust anchors of the CCADB. Mostly used by clients that want to interact
  /// with the internet.
  ///
  /// * Where did you get all this stuff? Internet!
  /// * Where did you get it for a dollar? Internet!
  /// * Where did you get heat vision? Internet!
  /// * Where did you get all these nice things? Internet!
  #[cfg(all(feature = "ccadb", feature = "std"))]
  #[inline]
  pub fn from_ccadb() -> Self {
    use crate::{collection::Vector, x509::CCADB};
    use std::sync::OnceLock;

    static TRUST_ANCHORS: OnceLock<Vector<CvTrustAnchor<'static>>> = OnceLock::new();
    let mut this = Self::new();
    this.trust_anchors = TRUST_ANCHORS.get_or_init(|| {
      let mut vector = Vector::new();
      for elem in CCADB {
        vector.push(CvTrustAnchor::_from_raw(*elem).unwrap()).unwrap();
      }
      vector
    });
    this
  }

  /// New instance from public and secret keys. Mostly used by servers.
  #[inline]
  pub const fn from_keys(public_key: &'any [u8], secret_key: &'any [u8]) -> Self {
    let mut this = Self::new();
    this.public_key = public_key;
    this.secret_key = secret_key;
    this
  }

  /// New instance from the given trust anchors. Mostly used by clients.
  #[inline]
  pub const fn from_trust_anchors(trust_anchor: &'any [CvTrustAnchor<'any>]) -> Self {
    let mut this = Self::new();
    this.trust_anchors = trust_anchor;
    this
  }

  /// Application-Layer Protocol Negotiation Extension
  ///
  /// <https://datatracker.ietf.org/doc/html/rfc7301>
  #[inline]
  pub const fn alpn(&mut self) -> &mut ArrayVectorU8<&'any [u8], MAX_ALPN_LEN> {
    &mut self.alpn.protocol_name_list
  }

  /// See [`CvPolicy`].
  #[inline]
  pub const fn cv_policy(&self) -> &CvPolicy<'_, '_> {
    &self.cv_policy
  }

  /// See [`CvPolicy`].
  #[inline]
  pub const fn trust_anchors(&self) -> &'any [CvTrustAnchor<'any>] {
    &self.trust_anchors
  }

  const fn new() -> Self {
    Self {
      alpn: Alpn { protocol_name_list: ArrayVector::new() },
      cipher_suites: ArrayVector::from_array_u8(CipherSuite::all()),
      cv_policy: CvPolicy::new(),
      key_shares: ArrayVector::from_array_u8([
        KeyShareEntry { group: NamedGroup::X25519, opaque: &[] },
        KeyShareEntry { group: NamedGroup::Secp256r1, opaque: &[] },
      ]),
      max_fragment_length: None,
      named_groups: ArrayVector::from_array_u8(NamedGroup::all()),
      offered_psks: OfferedPsks { offered_psks: ArrayVectorU8::new() },
      public_key: &[],
      secret_key: &[],
      server_name: None,
      signature_algorithms: ArrayVector::from_array_u8(SignatureScheme::PRIORITY),
      signature_algorithms_cert: ArrayVector::from_array_u8(SignatureScheme::PRIORITY),
      trust_anchors: &[],
    }
  }
}

impl<'any> Lease<TlsConfig<'any>> for TlsConfig<'any> {
  #[inline]
  fn lease(&self) -> &TlsConfig<'any> {
    self
  }
}

impl<'any> LeaseMut<TlsConfig<'any>> for TlsConfig<'any> {
  #[inline]
  fn lease_mut(&mut self) -> &mut TlsConfig<'any> {
    self
  }
}

impl Debug for TlsConfig<'_> {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("TlsConfig").finish()
  }
}
