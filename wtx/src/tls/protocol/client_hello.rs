// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.2

use crate::{
  calendar::DateTime,
  codec::{Decode, Encode},
  collections::{ArrayVectorCopy, ArrayVectorU8, Vector},
  crypto::SignatureTy,
  misc::{
    Lease, SingleTypeStorage,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  rng::CryptoRng,
  tls::{
    CipherSuite, MAX_KEY_SHARES_LEN, MaxFragmentLength, NamedGroup, TlsConfig, TlsError, TlsMode,
    de::De,
    misc::{u8_chunk, u16_chunk, u16_list},
    protocol::{
      alpn::Alpn,
      cert_types::CertTypes,
      extension::Extension,
      extension_ty::ExtensionTy,
      key_share_client_hello::KeyShareClientHello,
      key_share_entry::KeyShareEntry,
      named_group::NamedGroupAgreement,
      offered_psks::OfferedPsks,
      protocol_version::ProtocolVersion,
      protocol_versions::SupportedVersionsClient,
      psk_key_exchange_modes::{PskKeyExchangeMode, PskKeyExchangeModes},
      server_name_list::ServerNameList,
      signature_algorithms::SignatureAlgorithms,
      signature_algorithms_cert::SignatureAlgorithmsCert,
      supported_groups::SupportedGroups,
    },
    tls_certificate::TlsCertificate,
    tls_config::TlsConfigInner,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
  x509::CvPolicy,
};

#[derive(Debug)]
pub(crate) struct ClientHello<S, TC> {
  legacy_session_id: ArrayVectorCopy<u8, 32>,
  legacy_version: ProtocolVersion,
  psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  random: [u8; 32],
  secrets: S,
  supported_versions: SupportedVersionsClient,
  tls_config: TC,
}

impl<S, TC> ClientHello<S, TC> {
  pub(crate) fn new<RNG>(rng: &mut RNG, secrets: S, tls_config: TC) -> Self
  where
    RNG: CryptoRng,
  {
    Self {
      legacy_session_id: ArrayVectorCopy::from_array({
        let mut array = [0; 32];
        rng.fill_slice(&mut array);
        array
      }),
      legacy_version: ProtocolVersion::Tls12,
      psk_key_exchange_modes: Some(PskKeyExchangeModes::new(ArrayVectorU8::from_array([
        PskKeyExchangeMode::PskDheKe,
      ]))),
      random: {
        let mut array = [0u8; 32];
        rng.fill_slice(&mut array);
        array
      },
      secrets,
      supported_versions: SupportedVersionsClient::new(ArrayVectorCopy::from_array([
        ProtocolVersion::Tls13,
      ])),
      tls_config,
    }
  }

  pub(crate) fn legacy_session_id(&self) -> &ArrayVectorCopy<u8, 32> {
    &self.legacy_session_id
  }

  pub(crate) fn supported_versions(&self) -> &SupportedVersionsClient {
    &self.supported_versions
  }

  pub(crate) fn tls_config(&self) -> &TC {
    &self.tls_config
  }
}

impl<'de, TM> Decode<'de, De> for ClientHello<(), TlsConfigInner<&'de [u8], TM>>
where
  TM: TlsMode,
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidClientHelloLength;
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32] as Decode<'de, De>>::decode(dw)?;
    let legacy_session_id = u8_chunk(dw, err, |el| Ok(el.bytes()))?.try_into()?;
    let mut alpn = None;
    let mut cipher_suites = ArrayVectorCopy::new();
    let mut client_cert_types = None;
    let mut key_shares = ArrayVectorU8::new();
    let mut last_ty = None;
    let mut max_fragment_length = None;
    let mut named_groups = ArrayVectorCopy::new();
    let mut pre_shared_key = OfferedPsks { offered_psks: ArrayVectorU8::new() };
    let mut psk_key_exchange_modes = None;
    let mut server_cert_types = None;
    let mut server_name = None;
    let mut signature_algorithms = ArrayVectorCopy::new();
    let mut signature_algorithms_cert = ArrayVectorCopy::new();
    let mut supported_versions_opt = None;
    u16_list(&mut cipher_suites, dw, TlsError::InvalidCipherSuite)?;
    let _legacy_compression_methods @ [1, 0] = <[u8; 2] as Decode<'de, De>>::decode(dw)? else {
      return Err(TlsError::InvalidLegacyCompressionMethod.into());
    };
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        last_ty = Some(extension_ty);
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(
            &mut alpn,
            &mut client_cert_types,
            local_local_dw,
            extension_ty,
            &mut key_shares,
            &mut max_fragment_length,
            &mut named_groups,
            &mut pre_shared_key,
            &mut psk_key_exchange_modes,
            &mut server_cert_types,
            &mut server_name,
            &mut signature_algorithms,
            &mut signature_algorithms_cert,
            &mut supported_versions_opt,
          )
        })?;
      }
      Ok(())
    })?;
    let has_psk = !pre_shared_key.offered_psks.is_empty();
    if has_psk && last_ty != Some(ExtensionTy::PreSharedKey) {
      return Err(TlsError::BadPreKeyShare.into());
    }
    let Some(supported_versions) = supported_versions_opt else {
      return Err(TlsError::MissingSupportedVersions.into());
    };
    if signature_algorithms.is_empty() {
      return Err(TlsError::MissingSignatureAlgorithms.into());
    }
    if key_shares.is_empty() {
      return Err(TlsError::MissingKeyShares.into());
    }
    Ok(Self {
      legacy_session_id,
      legacy_version,
      psk_key_exchange_modes,
      random,
      secrets: (),
      supported_versions,
      tls_config: TlsConfigInner {
        alpn,
        cipher_suites,
        client_cert_types,
        cv_policy: CvPolicy::new(DateTime::default()),
        key_shares,
        max_fragment_length,
        named_groups,
        offered_psks: pre_shared_key,
        public_key: TlsCertificate::default(),
        secret_key: &[],
        server_cert_types,
        server_name,
        signature_algorithms,
        signature_algorithms_cert,
        trust_anchors: Vector::new(),
        mode: TM::default(),
      },
    })
  }
}

impl<TC, TM> Encode<De>
  for ClientHello<&'_ ArrayVectorU8<NamedGroupAgreement, MAX_KEY_SHARES_LEN>, TC>
where
  TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let _ = ew.buffer().extend_from_copyable_slices([
      u16::from(self.legacy_version).to_be_bytes().as_slice(),
      &self.random[..],
      &[self.legacy_session_id.len()][..],
      &self.legacy_session_id,
      u16::from(self.tls_config.lease().inner.cipher_suites.len().wrapping_mul(2))
        .to_be_bytes()
        .as_slice(),
      {
        let mut cipher_suites = ArrayVectorCopy::<_, { 2 * CipherSuite::len() }>::new();
        for cipher_suite in &self.tls_config.lease().inner.cipher_suites {
          cipher_suites.extend_from_copyable_slice(&u16::from(*cipher_suite).to_be_bytes())?;
        }
        cipher_suites
      }
      .as_slice(),
      &[1, 0], //legacy_compression_methods,
    ])?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      if let Some(elem) = &self.tls_config.lease().inner.alpn {
        Extension::new(ExtensionTy::ApplicationLayerProtocolNegotiation, elem).encode(local_ew)?;
      }
      if let Some(elem) = &self.tls_config.lease().inner.client_cert_types {
        Extension::new(ExtensionTy::ClientCertificateType, elem).encode(local_ew)?;
      }
      {
        let mut client_shares = ArrayVectorU8::<_, MAX_KEY_SHARES_LEN>::new();
        for (key_share, secret) in self.tls_config.lease().inner.key_shares.iter().zip(self.secrets)
        {
          client_shares.push((key_share.group, secret.public_key()?))?;
        }
        Extension::new(
          ExtensionTy::KeyShare,
          KeyShareClientHello {
            client_shares: ArrayVectorU8::from_iterator(
              client_shares
                .iter()
                .map(|(group, opaque)| KeyShareEntry { group: *group, opaque: opaque.as_ref() }),
            )?,
          },
        )
        .encode(local_ew)?;
      }
      if let Some(max_fragment_length) = self.tls_config.lease().inner.max_fragment_length {
        Extension::new(ExtensionTy::MaxFragmentLength, max_fragment_length).encode(local_ew)?;
      }
      if let Some(elem) = &self.tls_config.lease().inner.server_cert_types {
        Extension::new(ExtensionTy::ServerCertificateType, elem).encode(local_ew)?;
      }
      if let Some(name) = self.tls_config.lease().inner.server_name.as_ref() {
        Extension::new(ExtensionTy::ServerName, name).encode(local_ew)?;
      }
      Extension::new(
        ExtensionTy::SignatureAlgorithms,
        SignatureAlgorithms {
          signature_schemes: ArrayVectorCopy::from_iterator(
            self.tls_config.lease().inner.signature_algorithms.iter().copied(),
          )?,
        },
      )
      .encode(local_ew)?;
      Extension::new(
        ExtensionTy::SignatureAlgorithmsCert,
        SignatureAlgorithmsCert {
          supported_groups: self.tls_config.lease().inner.signature_algorithms_cert,
        },
      )
      .encode(local_ew)?;
      Extension::new(
        ExtensionTy::SupportedGroups,
        SupportedGroups { supported_groups: self.tls_config.lease().inner.named_groups },
      )
      .encode(local_ew)?;
      Extension::new(ExtensionTy::SupportedVersions, &self.supported_versions).encode(local_ew)?;
      if !self.tls_config.lease().inner.offered_psks.offered_psks.is_empty() {
        if let Some(modes) = &self.psk_key_exchange_modes {
          Extension::new(ExtensionTy::PskKeyExchangeModes, modes).encode(local_ew)?;
        }
        let data = &self.tls_config.lease().inner.offered_psks;
        Extension::new(ExtensionTy::PreSharedKey, data).encode(local_ew)?;
      }
      crate::Result::Ok(())
    })?;
    Ok(())
  }
}

fn duplicated_error(is_some: bool) -> crate::Result<()> {
  if is_some {
    return Err(TlsError::DuplicatedClientHelloParameters.into());
  }
  Ok(())
}

fn manage_extension<'de>(
  alpn: &mut Option<Alpn>,
  client_cert_types: &mut Option<CertTypes>,
  dw: &mut TlsDecodeWrapper<'de>,
  extension_ty: ExtensionTy,
  key_shares: &mut ArrayVectorU8<KeyShareEntry<&'de [u8]>, MAX_KEY_SHARES_LEN>,
  max_fragment_length: &mut Option<MaxFragmentLength>,
  named_groups: &mut ArrayVectorCopy<NamedGroup, { NamedGroup::len() }>,
  pre_shared_key: &mut OfferedPsks<&'de [u8]>,
  psk_key_exchange_modes: &mut Option<PskKeyExchangeModes>,
  server_cert_types: &mut Option<CertTypes>,
  server_name: &mut Option<ServerNameList>,
  signature_algorithms: &mut ArrayVectorCopy<SignatureTy, { SignatureTy::len() }>,
  signature_algorithms_cert: &mut ArrayVectorCopy<SignatureTy, { SignatureTy::len() }>,
  supported_versions_opt: &mut Option<SupportedVersionsClient>,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::ApplicationLayerProtocolNegotiation => {
      duplicated_error(alpn.is_some())?;
      *alpn = Some(Alpn::decode(dw)?);
    }
    ExtensionTy::ClientCertificateType => {
      duplicated_error(client_cert_types.is_some())?;
      *client_cert_types = Some(CertTypes::decode(dw)?);
    }
    ExtensionTy::MaxFragmentLength => {
      duplicated_error(max_fragment_length.is_some())?;
      *max_fragment_length = Some(MaxFragmentLength::decode(dw)?);
    }
    ExtensionTy::KeyShare => {
      duplicated_error(!key_shares.is_empty())?;
      *key_shares = KeyShareClientHello::<'_>::decode(dw)?.client_shares;
    }
    ExtensionTy::OidFilters => {
      return Err(TlsError::MismatchedExtension.into());
    }
    ExtensionTy::PskKeyExchangeModes => {
      duplicated_error(psk_key_exchange_modes.is_some())?;
      *psk_key_exchange_modes = Some(PskKeyExchangeModes::decode(dw)?);
    }
    ExtensionTy::PreSharedKey => {
      duplicated_error(!pre_shared_key.offered_psks.is_empty())?;
      *pre_shared_key = OfferedPsks::<&'de [u8]>::decode(dw)?;
    }
    ExtensionTy::ServerCertificateType => {
      duplicated_error(server_cert_types.is_some())?;
      *server_cert_types = Some(CertTypes::decode(dw)?);
    }
    ExtensionTy::ServerName => {
      duplicated_error(server_name.is_some())?;
      *server_name = Some(ServerNameList::decode(dw)?);
    }
    ExtensionTy::SignatureAlgorithms => {
      duplicated_error(!signature_algorithms.is_empty())?;
      *signature_algorithms = SignatureAlgorithms::decode(dw)?.signature_schemes;
    }
    ExtensionTy::SignatureAlgorithmsCert => {
      duplicated_error(!signature_algorithms_cert.is_empty())?;
      *signature_algorithms_cert = SignatureAlgorithmsCert::decode(dw)?.supported_groups;
    }
    ExtensionTy::SupportedGroups => {
      duplicated_error(!named_groups.is_empty())?;
      *named_groups = SupportedGroups::decode(dw)?.supported_groups;
    }
    ExtensionTy::SupportedVersions => {
      duplicated_error(supported_versions_opt.is_some())?;
      *supported_versions_opt = Some(SupportedVersionsClient::decode(dw)?);
    }
    ExtensionTy::CertificateAuthorities
    | ExtensionTy::Cookie
    | ExtensionTy::EarlyData
    | ExtensionTy::Heartbeat
    | ExtensionTy::Padding
    | ExtensionTy::PostHandshakeAuth
    | ExtensionTy::SignedCertificateTimestamp
    | ExtensionTy::StatusRequest
    | ExtensionTy::UseSrtp => {
      return Err(TlsError::UnsupportedExtension.into());
    }
  }
  Ok(())
}
