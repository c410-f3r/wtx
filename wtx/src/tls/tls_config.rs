use crate::{
  asn1::Asn1DecodeWrapperAux,
  calendar::{DateTime, Instant, Utc},
  codec::{Decode as _, DecodeWrapper, Pem},
  collections::{ArrayVectorCopy, ShortBoxSliceU16, SingleTypeStorage, Vector},
  misc::{Lease, LeaseMut},
  rng::CryptoRng,
  tls::{
    Alpn, CipherSuite, MaxFragmentLength, NamedGroup, PlaintextCtx, ServerNameList,
    SignatureScheme, TlsCtxSkInput, TlsCtxSkLoader, TrustedCtx, UnverifiedCtx,
    protocol::{
      signature_algorithms::SignatureAlgorithms,
      signature_algorithms_cert::SignatureAlgorithmsCert, supported_groups::SupportedGroups,
    },
  },
  x509::{Certificate, CvPolicy, CvTrustAnchor, PublicKeyTy},
};
use core::fmt::Debug;

/// TLS Configuration
///
/// This is a non-trivial structure that should be constructed only once in your application.
pub struct TlsConfig<TCX> {
  pub(crate) inner: TlsConfigInner<ShortBoxSliceU16<u8>, TCX>,
}

impl TlsConfig<PlaintextCtx> {
  /// Placeholder used in locals where data is expected to be unencrypted.
  #[inline]
  pub fn plaintext() -> Self {
    Self { inner: TlsConfigInner::new(PlaintextCtx::new(), DateTime::default()) }
  }
}

impl TlsConfig<TrustedCtx> {
  /// Set of filtered certificates from CCADB generally suitable for web scenarios.
  ///
  /// Fetches the current timestamp to verify certificates.
  #[cfg(feature = "ccadb")]
  #[inline]
  pub fn from_ccadb() -> crate::Result<Self> {
    let mut trust_anchors = Vector::new();
    for elem in crate::x509::CCADB {
      trust_anchors.push(CvTrustAnchor::_from_raw(*elem)?)?;
    }
    let mut this = Self::new(TrustedCtx::new())?;
    this.inner.trust_anchors = trust_anchors;
    Ok(this)
  }

  /// New instance from the given full X.509 trust anchors in PEM format. Mostly used by clients.
  ///
  /// Fetches the current timestamp to verify certificates
  #[inline]
  pub fn from_trust_anchors_pem<'bytes>(
    trust_anchors: impl IntoIterator<Item = &'bytes [u8]>,
  ) -> crate::Result<Self> {
    let mut this = Self::new(TrustedCtx::new())?;
    this.set_trust_anchors_pem(trust_anchors)?;
    Ok(this)
  }
}

impl TlsConfig<UnverifiedCtx> {
  /// Placeholder used in locals where data is expected to be unverified.
  #[inline]
  pub fn unverified() -> Self {
    Self { inner: TlsConfigInner::new(UnverifiedCtx::new(), DateTime::default()) }
  }
}

impl<TCX> TlsConfig<TCX>
where
  TCX: TlsCtxSkLoader,
{
  /// New instance from full X.509 public and secret keys in DER format. Mostly used by servers.
  ///
  /// Fetches the current timestamp to verify certificates
  #[inline]
  pub fn from_keys_der<'pk, 'sk, RNG, SK>(
    public_keys: impl IntoIterator<Item = &'pk [u8]>,
    rng: &mut RNG,
    secret_keys: SK,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
    SK: TlsCtxSkInput<TlsCtxSk = TCX>,
    TCX: TlsCtxSkLoader<SkInputDer<'sk> = SK>,
  {
    let mut this = Self::new(TCX::from_der(secret_keys, rng)?)?;
    this.set_public_keys_der(public_keys)?;
    Ok(this)
  }

  /// New instance from full X.509 public and secret keys in PEM format. Mostly used by servers.
  ///
  /// Fetches the current timestamp to verify certificates
  #[inline]
  pub fn from_keys_pem<'sk, RNG, SK>(
    public_keys: &[u8],
    rng: &mut RNG,
    secret_keys: SK,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
    SK: TlsCtxSkInput<TlsCtxSk = TCX>,
    TCX: TlsCtxSkLoader<SkInputPem<'sk> = SK>,
  {
    let mut this = Self::new(TCX::from_pem(secret_keys, rng)?)?;
    this.set_public_keys_pem(public_keys)?;
    Ok(this)
  }
}

impl<TCX> TlsConfig<TCX> {
  /// New instance that doesn't incorporate any initial certificate, which will likely make
  /// connections fail. However, it is still possible to add certificates using mutable methods.
  ///
  /// Fetches the current timestamp to verify certificates.
  #[inline]
  pub fn new(ctx: TCX) -> crate::Result<Self> {
    Ok(Self::from_validation_time(ctx, Instant::now_date_time()?))
  }

  /// Adjusts the validation time of [`CvPolicy`] that regulates certificate expiration.
  ///
  /// Taking aside [`Self::plaintext`], all other constructors implicitly fetch the current time.
  #[inline]
  pub fn from_validation_time(ctx: TCX, validation_time: DateTime<Utc>) -> Self {
    Self { inner: TlsConfigInner::new(ctx, validation_time) }
  }
}

impl<TCX> TlsConfig<TCX> {
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

  /// See [`CipherSuite`].
  #[inline]
  pub const fn cipher_suites(
    &self,
  ) -> &ArrayVectorCopy<CipherSuite, { CipherSuite::PRIORITY.len() }> {
    &self.inner.cipher_suites
  }

  /// Mutable version of [`Self::cipher_suites`].
  #[inline]
  pub const fn cipher_suites_mut(
    &mut self,
  ) -> &mut ArrayVectorCopy<CipherSuite, { CipherSuite::PRIORITY.len() }> {
    &mut self.inner.cipher_suites
  }

  /// See [`crate::tls::TlsCtx`].
  #[inline]
  pub const fn ctx(&self) -> &TCX {
    &self.inner.ctx
  }

  /// See [`CvPolicy`].
  #[inline]
  pub const fn cv_policy(&self) -> &CvPolicy<ShortBoxSliceU16<u8>> {
    &self.inner.cv_policy
  }

  /// Mutable version of [`Self::cv_policy`].
  #[inline]
  pub const fn cv_policy_mut(&mut self) -> &mut CvPolicy<ShortBoxSliceU16<u8>> {
    &mut self.inner.cv_policy
  }

  /// Maximum record size the local TLS instance is willing to accept from peers.
  ///
  /// * `Clients`: Servers must accept sent non-null parameters, otherwise the handshake will fail.
  /// * `Servers`: Forbids received values that are greater then non-nul parameters.
  ///
  /// If [`None`], defaults to `2^14`.
  #[inline]
  pub const fn max_fragment_length(&self) -> Option<MaxFragmentLength> {
    self.inner.max_fragment_length
  }

  /// Mutable version of [`Self::max_fragment_length`].
  ///
  /// If [`None`], defaults to `2^14`.
  #[inline]
  pub const fn max_fragment_length_mut(&mut self) -> &mut Option<MaxFragmentLength> {
    &mut self.inner.max_fragment_length
  }

  /// Maximum record size the local TLS instance can send to peers.
  ///
  /// If [`None`], defaults to `2^14`.
  #[inline]
  pub const fn max_fragment_length_send(&self) -> Option<MaxFragmentLength> {
    self.inner.max_fragment_length_send
  }

  /// Mutable version of [`Self::max_fragment_length_send`].
  ///
  /// If [`None`], defaults to `2^14`.
  #[inline]
  pub const fn max_fragment_length_send_mut(&mut self) -> &mut Option<MaxFragmentLength> {
    &mut self.inner.max_fragment_length_send
  }

  /// See [`ServerNameList`].
  #[inline]
  pub fn server_name(&self) -> &Option<ServerNameList> {
    &self.inner.server_name
  }

  /// Mutable version of [`Self::server_name`].
  #[inline]
  pub fn server_name_mut(&mut self) -> &mut Option<ServerNameList> {
    &mut self.inner.server_name
  }

  /// See [`ServerNameList`].
  #[inline]
  pub fn set_tls_mode<_TM>(self, value: _TM) -> TlsConfig<_TM> {
    TlsConfig {
      inner: TlsConfigInner {
        alpn: self.inner.alpn,
        cipher_suites: self.inner.cipher_suites,
        ctx: value,
        cv_policy: self.inner.cv_policy,
        max_fragment_length: self.inner.max_fragment_length,
        max_fragment_length_send: self.inner.max_fragment_length_send,
        public_key: self.inner.public_key,
        server_name: self.inner.server_name,
        signature_algorithms: self.inner.signature_algorithms,
        signature_algorithms_cert: self.inner.signature_algorithms_cert,
        supported_groups: self.inner.supported_groups,
        trust_anchors: self.inner.trust_anchors,
      },
    }
  }

  /// Converts X.509 certificates in DER format into public keys.
  #[inline]
  pub fn set_public_keys_der<'bytes>(
    &mut self,
    public_keys: impl IntoIterator<Item = &'bytes [u8]>,
  ) -> crate::Result<()> {
    self.inner.public_key.clear();
    for pk in public_keys {
      self.inner.public_key.push(public_key_from_der(pk)?)?;
    }
    Ok(())
  }

  /// Converts X.509 certificates in PEM format into public keys.
  #[inline]
  pub fn set_public_keys_pem(&mut self, public_keys: &[u8]) -> crate::Result<()> {
    self.inner.public_key = public_key_from_pem(&mut Vector::new(), public_keys)?;
    Ok(())
  }

  /// Converts X.509 certificates in PEM format into trust anchors.
  #[inline]
  pub fn set_trust_anchors_pem<'bytes>(
    &mut self,
    trust_anchors: impl IntoIterator<Item = &'bytes [u8]>,
  ) -> crate::Result<()> {
    let mut buffer = Vector::new();
    self.inner.trust_anchors.clear();
    for trust_anchor in trust_anchors {
      let certificate = Certificate::<&[u8]>::from_pem(&mut buffer, trust_anchor)?.0;
      self.inner.trust_anchors.push(CvTrustAnchor::from_certificate_ref(&certificate)?)?;
    }
    Ok(())
  }

  /// Every instance of [`TlsConfig`] is already pre-filled with a list of signature algorithms.
  ///
  /// See [`SignatureAlgorithms`].
  #[inline]
  pub fn signature_algorithms(&self) -> &SignatureAlgorithms {
    &self.inner.signature_algorithms
  }

  /// Mutable version of [`Self::signature_algorithms`].
  #[inline]
  pub fn signature_algorithms_mut(&mut self) -> &mut SignatureAlgorithms {
    &mut self.inner.signature_algorithms
  }

  /// See [`NamedGroup`].
  #[inline]
  pub const fn supported_groups(&self) -> &SupportedGroups {
    &self.inner.supported_groups
  }

  /// Mutable version of [`Self::supported_groups`].
  #[inline]
  pub const fn supported_groups_mut(&mut self) -> &mut SupportedGroups {
    &mut self.inner.supported_groups
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

impl<TCX> Lease<TlsConfig<TCX>> for TlsConfig<TCX> {
  #[inline]
  fn lease(&self) -> &TlsConfig<TCX> {
    self
  }
}

impl<TCX> LeaseMut<TlsConfig<TCX>> for TlsConfig<TCX> {
  #[inline]
  fn lease_mut(&mut self) -> &mut TlsConfig<TCX> {
    self
  }
}

impl<TCX> SingleTypeStorage for TlsConfig<TCX> {
  type Item = TCX;
}

impl<TCX> Debug for TlsConfig<TCX>
where
  TCX: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.inner.fmt(f)
  }
}

#[derive(Debug)]
pub(crate) struct TlsConfigInner<B, TCX> {
  pub(crate) alpn: Option<Alpn>,
  pub(crate) cipher_suites: ArrayVectorCopy<CipherSuite, { CipherSuite::PRIORITY.len() }>,
  pub(crate) ctx: TCX,
  pub(crate) cv_policy: CvPolicy<B>,
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) max_fragment_length_send: Option<MaxFragmentLength>,
  pub(crate) public_key: Vector<(PublicKeyTy, B)>,
  pub(crate) server_name: Option<ServerNameList>,
  pub(crate) signature_algorithms: SignatureAlgorithms,
  pub(crate) signature_algorithms_cert: Option<SignatureAlgorithmsCert>,
  pub(crate) supported_groups: SupportedGroups,
  pub(crate) trust_anchors: Vector<CvTrustAnchor<B>>,
}

impl<B, TCX> TlsConfigInner<B, TCX>
where
  B: Default,
{
  #[inline]
  fn new(ctx: TCX, validation_time: DateTime<Utc>) -> Self {
    Self {
      alpn: None,
      cipher_suites: ArrayVectorCopy::from_array(CipherSuite::PRIORITY),
      cv_policy: CvPolicy::new(validation_time),
      ctx,
      max_fragment_length: None,
      max_fragment_length_send: None,
      public_key: Vector::new(),
      server_name: None,
      signature_algorithms: SignatureAlgorithms::new(ArrayVectorCopy::from_array(
        SignatureScheme::PRIORITY,
      )),
      signature_algorithms_cert: Some(SignatureAlgorithmsCert::new(ArrayVectorCopy::from_array(
        SignatureScheme::PRIORITY,
      ))),
      supported_groups: SupportedGroups::new(ArrayVectorCopy::from_array(NamedGroup::PRIORITY)),
      trust_anchors: Vector::new(),
    }
  }
}

fn public_key_from_der<'de, B>(bytes: &'de [u8]) -> crate::Result<(PublicKeyTy, B)>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  let mut dw = DecodeWrapper::new(bytes, Asn1DecodeWrapperAux::default());
  let cert = &Certificate::<&[u8]>::decode(&mut dw)?;
  Ok((cert.try_into()?, bytes.try_into().map_err(Into::into)?))
}

fn public_key_from_pem<'de, B>(
  buffer: &'de mut Vector<u8>,
  bytes: &'de [u8],
) -> crate::Result<Vector<(PublicKeyTy, B)>>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  let pem = Pem::<_, 3>::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
  let mut certs = Vector::new();
  for (_, range) in pem.data {
    let cert_bytes = buffer.get(range.clone()).unwrap_or_default();
    let mut dw = DecodeWrapper::new(cert_bytes, Asn1DecodeWrapperAux::default());
    let cert = &Certificate::<&[u8]>::decode(&mut dw)?;
    certs.push((cert.try_into()?, cert_bytes.try_into().map_err(Into::into)?))?;
  }
  Ok(certs)
}
