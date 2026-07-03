use crate::{
  asn1::Asn1DecodeWrapperAux,
  codec::{Decode as _, DecodeWrapper},
  collections::{ArrayVector, ArrayVectorCopy, ArrayVectorU8, ShortBoxSliceU16, Vector},
  crypto::SignatureTy,
  misc::{Lease, LeaseMut, Pem, SingleTypeStorage},
  tls::{
    CipherSuite, MAX_KEY_SHARES_LEN, MaxFragmentLength, NamedGroup, TlsCertificateTy,
    TlsModePlainText,
    protocol::{
      alpn::Alpn, cert_types::CertTypes, key_share_entry::KeyShareEntry, offered_psks::OfferedPsks,
      server_name_list::ServerNameList,
    },
    tls_certificate::TlsCertificate,
  },
  x509::{Certificate, CvPolicy, CvTrustAnchor},
};
use core::fmt::Debug;

/// TLS Configuration
///
/// The is a non-trivial structure that should be constructed only once in your application.
pub struct TlsConfig<TM> {
  pub(crate) inner: TlsConfigInner<ShortBoxSliceU16<u8>, TM>,
}

impl TlsConfig<TlsModePlainText> {
  /// New instance that can't incorporate any certificate.
  #[inline]
  pub fn empty() -> Self {
    Self::new(TlsModePlainText::default())
  }
}

impl<TM> TlsConfig<TM> {
  /// New instance that doesn't incorporate any initial certificate, which will likely make
  /// connections fail. However, it is still possible to add certificates using mutable methods.
  #[inline]
  pub fn new(mode: TM) -> Self {
    Self { inner: TlsConfigInner::new(mode) }
  }

  /// Set of filtered certificates from CCADB generally suitable for web scenarios.
  #[cfg(feature = "ccadb")]
  #[inline]
  pub fn from_ccadb(mode: TM) -> crate::Result<Self> {
    let mut trust_anchors = Vector::new();
    for elem in crate::x509::CCADB {
      trust_anchors.push(CvTrustAnchor::_from_raw(*elem)?)?;
    }
    let mut this = Self::new(mode);
    this.inner.trust_anchors = trust_anchors;
    Ok(this)
  }

  /// New instance from full X.509 public and secret keys in PEM format. Mostly used by servers.
  #[inline]
  pub fn from_keys_pem(mode: TM, public_key: &[u8], secret_key: &[u8]) -> crate::Result<Self> {
    let mut this = Self::new(mode);
    let mut buffer = Vector::new();
    this.inner.public_key = tls_certificate(&mut buffer, public_key)?;
    buffer.clear();
    this.inner.secret_key = tls_certificate(&mut buffer, secret_key)?.x509;
    Ok(this)
  }

  /// New instance from the given full X.509 trust anchors in PEM format. Mostly used by clients.
  #[inline]
  pub fn from_trust_anchors_pem<'bytes>(
    mode: TM,
    trust_anchors: impl IntoIterator<Item = &'bytes [u8]>,
  ) -> crate::Result<Self> {
    let mut buffer = Vector::new();
    let mut vector = Vector::new();
    for trust_anchor in trust_anchors {
      let certificate = Certificate::<&[u8]>::from_pem(&mut buffer, trust_anchor)?.0;
      vector.push(CvTrustAnchor::from_certificate_ref(&certificate)?)?;
    }
    let mut this = Self::new(mode);
    this.inner.trust_anchors = vector;
    Ok(this)
  }

  /// See [`Alpn`].
  #[inline]
  pub const fn alpn(&self) -> &Option<Alpn> {
    &self.inner.alpn
  }

  /// Mutable version of [`Self::alpn`].
  #[inline]
  pub const fn alpn_mut(&mut self) -> &mut Option<Alpn> {
    &mut self.inner.alpn
  }

  /// See [`CvPolicy`].
  #[inline]
  pub const fn cv_policy(&self) -> &CvPolicy<ShortBoxSliceU16<u8>> {
    &self.inner.cv_policy
  }

  /// Maximum size of a TLS record
  ///
  /// Default to 2^24 - 1
  #[inline]
  pub const fn max_fragment_length(&self) -> Option<MaxFragmentLength> {
    self.inner.max_fragment_length
  }

  /// Mutable version of [`Self::max_fragment_length`].
  ///
  /// If [`None`], defaults to `2^24 -1`
  #[inline]
  pub const fn max_fragment_length_mut(&mut self) -> &mut Option<MaxFragmentLength> {
    &mut self.inner.max_fragment_length
  }

  /// See [`crate::tls::TlsMode`].
  #[inline]
  pub const fn mode(&self) -> &TM {
    &self.inner.mode
  }

  /// **For clients**: Tells the server which types of certificates it can send.
  /// **For servers**: The types of certificates it can receive, aborting the handshake if there
  ///                  isn't a match with clients. Picks the first compatible type.
  ///
  /// If empty, the handshake will default to X.509.
  #[inline]
  #[must_use]
  pub fn set_client_cert_types(mut self, value: ArrayVectorCopy<TlsCertificateTy, 2>) -> Self {
    self.inner.client_cert_types = CertTypes(value);
    self
  }

  /// See [`crate::tls::TlsMode`].
  #[inline]
  pub fn set_mode<_TM>(self, value: _TM) -> TlsConfig<_TM> {
    TlsConfig {
      inner: TlsConfigInner {
        alpn: self.inner.alpn,
        cipher_suites: self.inner.cipher_suites,
        client_cert_types: self.inner.client_cert_types,
        cv_policy: self.inner.cv_policy,
        key_shares: self.inner.key_shares,
        max_fragment_length: self.inner.max_fragment_length,
        named_groups: self.inner.named_groups,
        offered_psks: self.inner.offered_psks,
        public_key: self.inner.public_key,
        secret_key: self.inner.secret_key,
        server_cert_types: self.inner.server_cert_types,
        server_name: self.inner.server_name,
        signature_algorithms_cert: self.inner.signature_algorithms_cert,
        signature_algorithms: self.inner.signature_algorithms,
        trust_anchors: self.inner.trust_anchors,
        mode: value,
      },
    }
  }

  /// **For clients**: Tells the server which types of certificates it can receive.
  /// **For servers**: The types of certificates it can send, aborting the handshake if there
  ///                  isn't a match with clients. Picks the first compatible type.
  ///
  /// If empty, the handshake will default to X.509.
  #[inline]
  #[must_use]
  pub fn set_server_cert_types(mut self, value: ArrayVectorCopy<TlsCertificateTy, 2>) -> Self {
    self.inner.server_cert_types = CertTypes(value);
    self
  }

  /// See [`CvTrustAnchor`].
  #[inline]
  pub fn trust_anchors(&self) -> &[CvTrustAnchor<ShortBoxSliceU16<u8>>] {
    &self.inner.trust_anchors
  }

  /// Mutable version of [`Self::trust_anchors`].
  #[inline]
  pub fn trust_anchors_mut(&mut self) -> &mut Vector<CvTrustAnchor<ShortBoxSliceU16<u8>>> {
    &mut self.inner.trust_anchors
  }
}

impl<TM> Lease<TlsConfig<TM>> for TlsConfig<TM> {
  #[inline]
  fn lease(&self) -> &TlsConfig<TM> {
    self
  }
}

impl<TM> LeaseMut<TlsConfig<TM>> for TlsConfig<TM> {
  #[inline]
  fn lease_mut(&mut self) -> &mut TlsConfig<TM> {
    self
  }
}

impl<TM> SingleTypeStorage for TlsConfig<TM> {
  type Item = TM;
}

impl<TM> Debug for TlsConfig<TM> {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("TlsConfig").finish()
  }
}

#[derive(Clone)]
pub(crate) struct TlsConfigInner<B, TM> {
  pub(crate) alpn: Option<Alpn>,
  pub(crate) cipher_suites: ArrayVectorCopy<CipherSuite, { CipherSuite::len() }>,
  pub(crate) client_cert_types: CertTypes,
  pub(crate) cv_policy: CvPolicy<B>,
  pub(crate) key_shares: ArrayVectorU8<KeyShareEntry<B>, MAX_KEY_SHARES_LEN>,
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) named_groups: ArrayVectorCopy<NamedGroup, { NamedGroup::len() }>,
  pub(crate) offered_psks: OfferedPsks<B>,
  pub(crate) public_key: TlsCertificate<B>,
  pub(crate) secret_key: B,
  pub(crate) server_cert_types: CertTypes,
  pub(crate) server_name: Option<ServerNameList<B>>,
  pub(crate) signature_algorithms_cert: ArrayVectorCopy<SignatureTy, { SignatureTy::len() }>,
  pub(crate) signature_algorithms: ArrayVectorCopy<SignatureTy, { SignatureTy::len() }>,
  pub(crate) trust_anchors: Vector<CvTrustAnchor<B>>,
  pub(crate) mode: TM,
}

impl<B, TM> TlsConfigInner<B, TM>
where
  B: Default,
{
  #[inline]
  fn new(mode: TM) -> Self {
    Self {
      alpn: None,
      client_cert_types: CertTypes::default(),
      cipher_suites: ArrayVectorCopy::from_array(CipherSuite::all()),
      cv_policy: CvPolicy::new(),
      key_shares: ArrayVector::from_array([
        KeyShareEntry { group: NamedGroup::X25519, opaque: B::default() },
        KeyShareEntry { group: NamedGroup::Secp256r1, opaque: B::default() },
      ]),
      max_fragment_length: None,
      named_groups: ArrayVectorCopy::from_array(NamedGroup::all()),
      offered_psks: OfferedPsks { offered_psks: ArrayVectorU8::new() },
      public_key: TlsCertificate::default(),
      secret_key: B::default(),
      server_cert_types: CertTypes::default(),
      server_name: None,
      signature_algorithms: ArrayVectorCopy::from_array(SignatureTy::TLS_PRIORITY),
      signature_algorithms_cert: ArrayVectorCopy::from_array(SignatureTy::TLS_PRIORITY),
      trust_anchors: Vector::new(),
      mode,
    }
  }
}

fn tls_certificate<'de, B>(
  buffer: &'de mut Vector<u8>,
  bytes: &'de [u8],
) -> crate::Result<TlsCertificate<B>>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  let pem = Pem::<_, 1>::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
  let [(_label, range)] = pem.data.into_inner()?;
  let cert_bytes = buffer.get(range.clone()).unwrap_or_default();
  let mut dw = DecodeWrapper::new(cert_bytes, Asn1DecodeWrapperAux::default());
  let _cert = Certificate::<&[u8]>::decode(&mut dw)?;
  let spki = dw.decode_aux.spki(dw.bytes).unwrap_or_default();
  Ok(TlsCertificate {
    raw_public_key: spki.try_into().map_err(Into::into)?,
    x509: cert_bytes.try_into().map_err(Into::into)?,
  })
}
