use crate::{
  asn1::{Asn1DecodeWrapperAux, Pkcs8},
  calendar::{DateTime, Instant, Utc},
  codec::{Decode as _, DecodeWrapper},
  collections::{ArrayVectorCopy, ShortBoxSliceU16, Vector},
  crypto::SignatureTy,
  misc::{Lease, LeaseMut, Pem, Secret, SecretContext, SensitiveBytes, SingleTypeStorage},
  rng::CryptoRng,
  tls::{
    Alpn, CipherSuite, MaxFragmentLength, NamedGroup, ServerNameList, TlsModePlainText,
    protocol::{
      signature_algorithms::SignatureAlgorithms,
      signature_algorithms_cert::SignatureAlgorithmsCert, supported_groups::SupportedGroups,
    },
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
  /// Placeholder used in locals where data is expected to be unencrypted.
  #[inline]
  pub fn plaintext() -> Self {
    Self::new(TlsModePlainText::default(), DateTime::default())
  }
}

impl<TM> TlsConfig<TM> {
  /// New instance that doesn't incorporate any initial certificate, which will likely make
  /// connections fail. However, it is still possible to add certificates using mutable methods.
  #[inline]
  pub fn new(mode: TM, validation_time: DateTime<Utc>) -> Self {
    Self { inner: TlsConfigInner::new(mode, validation_time) }
  }

  /// Set of filtered certificates from CCADB generally suitable for web scenarios.
  ///
  /// Fetches the current timestamp to verify certificates
  #[cfg(feature = "ccadb")]
  #[inline]
  pub fn from_ccadb(mode: TM) -> crate::Result<Self> {
    let mut trust_anchors = Vector::new();
    for elem in crate::x509::CCADB {
      trust_anchors.push(CvTrustAnchor::_from_raw(*elem)?)?;
    }
    let mut this = Self::new(mode, Instant::now_date_time()?);
    this.inner.trust_anchors = trust_anchors;
    Ok(this)
  }

  /// New instance from full X.509 public and secret keys in DER format. Mostly used by servers.
  ///
  /// Fetches the current timestamp to verify certificates
  #[inline]
  pub fn from_keys_der<'pk, RNG>(
    mode: TM,
    public_keys: impl IntoIterator<Item = &'pk [u8]>,
    rng: &mut RNG,
    (secret_context, secret_key): (SecretContext, &mut [u8]),
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut this = Self::new(mode, Instant::now_date_time()?);
    let mut public_key = Vector::new();
    for pk in public_keys {
      public_key.push(public_key_from_der(pk)?)?;
    }
    this.inner.public_key = public_key;
    this.inner.secret_key = Secret::new(secret_key, rng, secret_context)?;
    Ok(this)
  }

  /// New instance from full X.509 public and secret keys in PEM format. Mostly used by servers.
  ///
  /// Fetches the current timestamp to verify certificates
  #[inline]
  pub fn from_keys_pem<RNG>(
    mode: TM,
    public_key: &[u8],
    rng: &mut RNG,
    (secret_context, secret_key): (SecretContext, &mut [u8]),
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut this = Self::new(mode, Instant::now_date_time()?);
    let mut buffer = Vector::new();
    this.inner.public_key = public_key_from_pem(&mut buffer, public_key)?;
    buffer.clear();
    let secret_key_wrapper = SensitiveBytes::new(secret_key);
    this.inner.secret_key = {
      let pem = Pem::<_, 1>::decode(&mut DecodeWrapper::new(&secret_key_wrapper, &mut buffer))?;
      let [(_label, range)] = pem.data.into_inner()?;
      let data = buffer.get(range.clone()).unwrap_or_default();
      let mut dw = DecodeWrapper::new(data, Asn1DecodeWrapperAux::default());
      let _pkcs8 = Pkcs8::<&[u8]>::decode(&mut dw)?;
      Secret::new(buffer.get_mut(range).unwrap_or_default(), rng, secret_context)?
    };
    Ok(this)
  }

  /// New instance from the given full X.509 trust anchors in PEM format. Mostly used by clients.
  ///
  /// Fetches the current timestamp to verify certificates
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
    let mut this = Self::new(mode, Instant::now_date_time()?);
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

  /// See [`CipherSuite`].
  #[inline]
  pub const fn cipher_suites(&self) -> &ArrayVectorCopy<CipherSuite, { CipherSuite::ALL.len() }> {
    &self.inner.cipher_suites
  }

  /// Mutable version of [`Self::cipher_suites`].
  #[inline]
  pub const fn cipher_suites_mut(
    &mut self,
  ) -> &mut ArrayVectorCopy<CipherSuite, { CipherSuite::ALL.len() }> {
    &mut self.inner.cipher_suites
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

  /// See [`ServerNameList`].
  #[inline]
  pub fn server_name_mut(&mut self) -> &mut Option<ServerNameList> {
    &mut self.inner.server_name
  }

  /// See [`NamedGroup`].
  #[inline]
  pub const fn supported_groups(&self) -> &ArrayVectorCopy<NamedGroup, { NamedGroup::len() }> {
    &self.inner.supported_groups.named_group_list
  }

  /// Mutable version of [`Self::supported_groups`].
  #[inline]
  pub const fn supported_groups_mut(
    &mut self,
  ) -> &mut ArrayVectorCopy<NamedGroup, { NamedGroup::len() }> {
    &mut self.inner.supported_groups.named_group_list
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

pub(crate) struct TlsConfigInner<B, TM> {
  pub(crate) alpn: Option<Alpn>,
  pub(crate) cipher_suites: ArrayVectorCopy<CipherSuite, { CipherSuite::ALL.len() }>,
  pub(crate) cv_policy: CvPolicy<B>,
  pub(crate) max_fragment_length: Option<MaxFragmentLength>,
  pub(crate) public_key: Vector<(SignatureTy, B)>,
  pub(crate) secret_key: Secret,
  pub(crate) server_name: Option<ServerNameList>,
  pub(crate) signature_algorithms: SignatureAlgorithms,
  pub(crate) signature_algorithms_cert: Option<SignatureAlgorithmsCert>,
  pub(crate) supported_groups: SupportedGroups,
  pub(crate) trust_anchors: Vector<CvTrustAnchor<B>>,
  pub(crate) mode: TM,
}

impl<B, TM> TlsConfigInner<B, TM>
where
  B: Default,
{
  #[inline]
  fn new(mode: TM, validation_time: DateTime<Utc>) -> Self {
    Self {
      alpn: None,
      cipher_suites: ArrayVectorCopy::from_array(CipherSuite::ALL),
      cv_policy: CvPolicy::new(validation_time),
      max_fragment_length: None,
      public_key: Vector::new(),
      secret_key: Secret::default(),
      server_name: None,
      signature_algorithms: SignatureAlgorithms::new(ArrayVectorCopy::from_array(
        SignatureTy::TLS_PRIORITY,
      )),
      signature_algorithms_cert: Some(SignatureAlgorithmsCert::new(ArrayVectorCopy::from_array(
        SignatureTy::TLS_PRIORITY,
      ))),
      supported_groups: SupportedGroups::new(ArrayVectorCopy::from_array(NamedGroup::all())),
      trust_anchors: Vector::new(),
      mode,
    }
  }
}

fn public_key_from_der<'de, B>(bytes: &'de [u8]) -> crate::Result<(SignatureTy, B)>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  let mut dw = DecodeWrapper::new(bytes, Asn1DecodeWrapperAux::default());
  let cert = Certificate::<&[u8]>::decode(&mut dw)?;
  let spki = &cert.tbs_certificate().subject_public_key_info;
  Ok((spki.try_into()?, bytes.try_into().map_err(Into::into)?))
}

fn public_key_from_pem<'de, B>(
  buffer: &'de mut Vector<u8>,
  bytes: &'de [u8],
) -> crate::Result<Vector<(SignatureTy, B)>>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  let pem = Pem::<_, 3>::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
  let mut certs = Vector::new();
  for (_, range) in pem.data {
    let cert_bytes = buffer.get(range.clone()).unwrap_or_default();
    let mut dw = DecodeWrapper::new(cert_bytes, Asn1DecodeWrapperAux::default());
    let cert = Certificate::<&[u8]>::decode(&mut dw)?;
    let spki = &cert.tbs_certificate().subject_public_key_info;
    certs.push((spki.try_into()?, cert_bytes.try_into().map_err(Into::into)?))?;
  }
  Ok(certs)
}
