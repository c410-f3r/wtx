// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.2

use crate::{
  codec::{Decode, Encode},
  collections::{ArrayVectorCopy, ArrayVectorU8, Vector},
  misc::{
    Lease, SingleTypeStorage,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  rng::CryptoRng,
  tls::{
    CipherSuite, MAX_KEY_SHARES_LEN, MaxFragmentLength, TlsConfig, TlsError, TlsMode,
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
      protocol_versions::SupportedVersions,
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
  legacy_compression_methods: [u8; 2],
  legacy_session_id: ArrayVectorCopy<u8, 32>,
  legacy_version: ProtocolVersion,
  psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  random: [u8; 32],
  secrets: S,
  supported_versions: SupportedVersions,
  tls_config: TC,
}

impl<S, TC> ClientHello<S, TC> {
  pub(crate) fn new<RNG>(rng: &mut RNG, secrets: S, tls_config: TC) -> Self
  where
    RNG: CryptoRng,
  {
    Self {
      legacy_compression_methods: [1, 0],
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
      supported_versions: SupportedVersions::new(ArrayVectorCopy::from_array([
        ProtocolVersion::Tls13,
      ])),
      tls_config,
    }
  }

  pub(crate) fn legacy_session_id(&self) -> &ArrayVectorCopy<u8, 32> {
    &self.legacy_session_id
  }

  pub(crate) fn tls_config(&self) -> &TC {
    &self.tls_config
  }
}

impl<'de, TM> Decode<'de, De> for ClientHello<(), TlsConfigInner<&'de [u8], TM>>
where
  TM: TlsMode,
{
  #[expect(clippy::too_many_lines, reason = "enum is big")]
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32] as Decode<'de, De>>::decode(dw)?;
    let legacy_session_id =
      u8_chunk(dw, TlsError::InvalidLegacySessionId, |el| Ok(el.bytes()))?.try_into()?;
    let mut alpn = Alpn { protocol_name_list: ArrayVectorCopy::new() };
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
    let legacy_compression_methods = <[u8; 2] as Decode<'de, De>>::decode(dw)?;
    u16_chunk(dw, TlsError::InvalidClientHelloLength, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *local_dw;
          ExtensionTy::decode(tmp_bytes)?
        };
        last_ty = Some(extension_ty);
        match extension_ty {
          ExtensionTy::ApplicationLayerProtocolNegotiation => {
            duplicated_error(!alpn.protocol_name_list.is_empty())?;
            alpn = Extension::<Alpn>::decode(local_dw)?.into_data();
          }
          ExtensionTy::ClientCertificateType => {
            duplicated_error(client_cert_types.is_some())?;
            client_cert_types = Some(Extension::<CertTypes>::decode(local_dw)?.into_data());
          }
          ExtensionTy::MaxFragmentLength => {
            duplicated_error(max_fragment_length.is_some())?;
            max_fragment_length =
              Some(Extension::<MaxFragmentLength>::decode(local_dw)?.into_data());
          }
          ExtensionTy::KeyShare => {
            duplicated_error(!key_shares.is_empty())?;
            key_shares =
              Extension::<KeyShareClientHello<'_>>::decode(local_dw)?.into_data().client_shares;
          }
          ExtensionTy::OidFilters => {
            return Err(TlsError::MismatchedExtension.into());
          }
          ExtensionTy::PskKeyExchangeModes => {
            duplicated_error(psk_key_exchange_modes.is_some())?;
            psk_key_exchange_modes =
              Some(Extension::<PskKeyExchangeModes>::decode(local_dw)?.into_data());
          }
          ExtensionTy::PreSharedKey => {
            duplicated_error(!pre_shared_key.offered_psks.is_empty())?;
            pre_shared_key = Extension::<OfferedPsks<&'de [u8]>>::decode(local_dw)?.into_data();
          }
          ExtensionTy::ServerCertificateType => {
            duplicated_error(server_cert_types.is_some())?;
            server_cert_types = Some(Extension::<CertTypes>::decode(local_dw)?.into_data());
          }
          ExtensionTy::ServerName => {
            duplicated_error(server_name.is_some())?;
            server_name =
              Some(Extension::<ServerNameList<&'de [u8]>>::decode(local_dw)?.into_data());
          }
          ExtensionTy::SignatureAlgorithms => {
            duplicated_error(!signature_algorithms.is_empty())?;
            signature_algorithms =
              Extension::<SignatureAlgorithms>::decode(local_dw)?.into_data().signature_schemes;
          }
          ExtensionTy::SignatureAlgorithmsCert => {
            duplicated_error(!signature_algorithms_cert.is_empty())?;
            signature_algorithms_cert =
              Extension::<SignatureAlgorithmsCert>::decode(local_dw)?.into_data().supported_groups;
          }
          ExtensionTy::SupportedGroups => {
            duplicated_error(!named_groups.is_empty())?;
            named_groups =
              Extension::<SupportedGroups>::decode(local_dw)?.into_data().supported_groups;
          }
          ExtensionTy::SupportedVersions => {
            duplicated_error(supported_versions_opt.is_some())?;
            supported_versions_opt =
              Some(Extension::<SupportedVersions>::decode(local_dw)?.into_data());
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
      }
      Ok(())
    })?;
    let has_psk = !pre_shared_key.offered_psks.is_empty() || psk_key_exchange_modes.is_some();
    if has_psk && last_ty != Some(ExtensionTy::PreSharedKey) {
      return Err(TlsError::BadPreKeyShare.into());
    }
    let Some(supported_versions) = supported_versions_opt else {
      return Err(TlsError::MissingSupportedVersions.into());
    };
    let [ProtocolVersion::Tls13] = supported_versions.versions.as_slice() else {
      return Err(TlsError::UnsupportedTlsVersion.into());
    };
    if signature_algorithms.is_empty() {
      return Err(TlsError::MissingSignatureAlgorithms.into());
    }
    if key_shares.is_empty() {
      return Err(TlsError::MissingKeyShares.into());
    }
    Ok(Self {
      legacy_compression_methods,
      legacy_session_id,
      legacy_version,
      psk_key_exchange_modes,
      random,
      secrets: (),
      supported_versions,
      tls_config: TlsConfigInner {
        alpn,
        cipher_suites,
        client_cert_types: client_cert_types.unwrap_or_default(),
        cv_policy: CvPolicy::default(),
        key_shares,
        max_fragment_length,
        named_groups,
        offered_psks: pre_shared_key,
        public_key: TlsCertificate::default(),
        secret_key: &[],
        server_cert_types: server_cert_types.unwrap_or_default(),
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
      &[32],
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
      &self.legacy_compression_methods,
    ])?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      if !self.tls_config.lease().inner.alpn.protocol_name_list.is_empty() {
        Extension::new(
          ExtensionTy::ApplicationLayerProtocolNegotiation,
          &self.tls_config.lease().inner.alpn,
        )
        .encode(local_ew)?;
      }
      Extension::new(
        ExtensionTy::ClientCertificateType,
        &self.tls_config.lease().inner.client_cert_types,
      )
      .encode(local_ew)?;
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
      Extension::new(ExtensionTy::PskKeyExchangeModes, &self.psk_key_exchange_modes)
        .encode(local_ew)?;
      Extension::new(
        ExtensionTy::ServerCertificateType,
        &self.tls_config.lease().inner.server_cert_types,
      )
      .encode(local_ew)?;
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
        Extension::new(ExtensionTy::PreSharedKey, &self.tls_config.lease().inner.offered_psks)
          .encode(local_ew)?;
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
