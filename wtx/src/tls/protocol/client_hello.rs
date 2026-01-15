// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.2

use crate::{
  collection::{ArrayVector, ArrayVectorU8},
  de::{Decode, Encode},
  misc::{
    Lease, SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  rng::CryptoRng,
  tls::{
    KEY_SHARES_LEN, MaxFragmentLength, TlsError,
    cipher_suite::CipherSuiteTy,
    de::De,
    misc::{u8_chunk, u16_chunk, u16_list},
    protocol::{
      client_hello_extension::ClientHelloExtension,
      client_hello_extension_ty::ClientHelloExtensionTy,
      ephemeral_secret_key::EphemeralSecretKey,
      key_share_client_hello::KeyShareClientHello,
      key_share_entry::KeyShareEntry,
      offered_psks::OfferedPsks,
      protocol_version::ProtocolVersion,
      protocol_versions::SupportedVersions,
      psk_key_exchange_modes::{PskKeyExchangeMode, PskKeyExchangeModes},
      server_name_list::ServerNameList,
      signature_algorithms::SignatureAlgorithms,
      signature_algorithms_cert::SignatureAlgorithmsCert,
      supported_groups::SupportedGroups,
    },
    tls_config::TlsConfigInner,
  },
};

#[derive(Debug)]
pub(crate) struct ClientHello<ES, TC> {
  legacy_version: ProtocolVersion,
  random: [u8; 32],
  legacy_session_id: ArrayVectorU8<u8, 32>,
  legacy_compression_methods: [u8; 2],
  secrets: ArrayVectorU8<ES, KEY_SHARES_LEN>,
  supported_versions: SupportedVersions,
  psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  tls_config: TC,
}

impl<'any, ES, TC> ClientHello<ES, TC>
where
  ES: EphemeralSecretKey,
  TC: Lease<TlsConfigInner<'any>>,
{
  pub(crate) fn new<RNG>(rng: &mut RNG, tls_config: TC) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut secrets = ArrayVectorU8::new();
    for key_share in &tls_config.lease().key_shares {
      secrets.push(ES::random(key_share.group, rng)?)?;
    }
    Ok(Self {
      legacy_version: ProtocolVersion::Tls12,
      random: {
        let mut array = [0u8; 32];
        rng.fill_slice(&mut array);
        array
      },
      legacy_session_id: ArrayVectorU8::from_array({
        let mut array = [0; 32];
        rng.fill_slice(&mut array[0..4]);
        array
      }),
      legacy_compression_methods: [1, 0],
      secrets,
      supported_versions: SupportedVersions::new(ArrayVectorU8::from_array([
        ProtocolVersion::Tls13,
      ])),
      psk_key_exchange_modes: Some(PskKeyExchangeModes::new(ArrayVectorU8::from_array([
        PskKeyExchangeMode::PskDheKe,
      ]))),
      tls_config,
    })
  }

  pub(crate) fn into_secrets(self) -> ArrayVectorU8<ES, KEY_SHARES_LEN> {
    self.secrets
  }

  pub(crate) fn legacy_session_id(&self) -> &ArrayVectorU8<u8, 32> {
    &self.legacy_session_id
  }

  pub(crate) fn tls_config(&self) -> &TlsConfigInner<'any> {
    self.tls_config.lease()
  }
}

impl<'de, ES> Decode<'de, De> for ClientHello<ES, TlsConfigInner<'de>>
where
  ES: EphemeralSecretKey,
{
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32]>::decode(dw)?;
    let legacy_session_id =
      u8_chunk(dw, TlsError::InvalidLegacySessionId, |el| Ok(*el))?.try_into()?;
    let mut cipher_suites = ArrayVectorU8::new();
    u16_list(&mut cipher_suites, dw, TlsError::InvalidCipherSuite)?;
    let legacy_compression_methods = <[u8; 2]>::decode(dw)?;
    let mut key_shares = ArrayVector::new();
    let mut max_fragment_length = None;
    let mut named_groups = ArrayVector::new();
    let mut pre_shared_key = OfferedPsks { offered_psks: ArrayVectorU8::new() };
    let mut psk_key_exchange_modes = None;
    let mut server_name = None;
    let mut signature_algorithms = ArrayVector::new();
    let mut signature_algorithms_cert = ArrayVector::new();
    let mut supported_versions_opt = None;
    let mut last_ty = None;
    u16_chunk(dw, TlsError::InvalidClientHelloLength, |bytes| {
      while !bytes.is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *bytes;
          ClientHelloExtensionTy::decode(tmp_bytes)?
        };
        last_ty = Some(extension_ty);
        match extension_ty {
          ClientHelloExtensionTy::ServerName => {
            duplicated_error(server_name.is_some())?;
            server_name = Some(ClientHelloExtension::<ServerNameList>::decode(bytes)?.into_data());
          }
          ClientHelloExtensionTy::MaxFragmentLength => {
            duplicated_error(max_fragment_length.is_some())?;
            max_fragment_length =
              Some(ClientHelloExtension::<MaxFragmentLength>::decode(bytes)?.into_data());
          }
          ClientHelloExtensionTy::StatusRequest => {}
          ClientHelloExtensionTy::SupportedGroups => {
            duplicated_error(!named_groups.is_empty())?;
            named_groups =
              ClientHelloExtension::<SupportedGroups>::decode(bytes)?.into_data().supported_groups;
          }
          ClientHelloExtensionTy::SignatureAlgorithms => {
            duplicated_error(!signature_algorithms.is_empty())?;
            signature_algorithms = ClientHelloExtension::<SignatureAlgorithms>::decode(bytes)?
              .into_data()
              .signature_schemes;
          }
          ClientHelloExtensionTy::UseSrtp => {}
          ClientHelloExtensionTy::Heartbeat => {}
          ClientHelloExtensionTy::ApplicationLayerProtocolNegotiation => {}
          ClientHelloExtensionTy::SignedCertificateTimestamp => {}
          ClientHelloExtensionTy::ClientCertificateType => {}
          ClientHelloExtensionTy::ServerCertificateType => {}
          ClientHelloExtensionTy::Padding => {}
          ClientHelloExtensionTy::PreSharedKey => {
            duplicated_error(!pre_shared_key.offered_psks.is_empty())?;
            pre_shared_key = ClientHelloExtension::<OfferedPsks<'_>>::decode(bytes)?.into_data();
          }
          ClientHelloExtensionTy::EarlyData => {}
          ClientHelloExtensionTy::SupportedVersions => {
            duplicated_error(supported_versions_opt.is_some())?;
            supported_versions_opt =
              Some(ClientHelloExtension::<SupportedVersions>::decode(bytes)?.into_data());
          }
          ClientHelloExtensionTy::Cookie => {}
          ClientHelloExtensionTy::PskKeyExchangeModes => {
            duplicated_error(psk_key_exchange_modes.is_some())?;
            psk_key_exchange_modes =
              Some(ClientHelloExtension::<PskKeyExchangeModes>::decode(bytes)?.into_data());
          }
          ClientHelloExtensionTy::CertificateAuthorities => {}
          ClientHelloExtensionTy::PostHandshakeAuth => {}
          ClientHelloExtensionTy::SignatureAlgorithmsCert => {
            duplicated_error(!signature_algorithms_cert.is_empty())?;
            signature_algorithms_cert =
              ClientHelloExtension::<SignatureAlgorithmsCert>::decode(bytes)?
                .into_data()
                .supported_groups;
          }
          ClientHelloExtensionTy::KeyShare => {
            duplicated_error(!key_shares.is_empty())?;
            key_shares = ClientHelloExtension::<KeyShareClientHello<'_>>::decode(bytes)?
              .into_data()
              .client_shares;
          }
        }
      }
      Ok(())
    })?;
    let has_psk = !pre_shared_key.offered_psks.is_empty() || psk_key_exchange_modes.is_some();
    if has_psk && last_ty != Some(ClientHelloExtensionTy::PreSharedKey) {
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
      secrets: ArrayVectorU8::new(),
      supported_versions,
      tls_config: TlsConfigInner {
        root_ca: None,
        certificate: None,
        cipher_suites,
        key_shares,
        max_fragment_length,
        named_groups,
        offered_psks: pre_shared_key,
        secret_key: &[],
        server_name,
        signature_algorithms,
        signature_algorithms_cert,
      },
    })
  }
}

impl<'any, ES, TC> Encode<De> for ClientHello<ES, TC>
where
  ES: EphemeralSecretKey,
  TC: Lease<TlsConfigInner<'any>>,
{
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slices([
      u16::from(self.legacy_version).to_be_bytes().as_slice(),
      &self.random[..],
      &self.legacy_session_id,
      u16::try_from(self.tls_config.lease().cipher_suites.len().wrapping_mul(2))
        .unwrap_or_default()
        .to_be_bytes()
        .as_slice(),
      {
        let mut cipher_suites = ArrayVectorU8::<_, { 2 * CipherSuiteTy::len() }>::new();
        for cipher_suite in &self.tls_config.lease().cipher_suites {
          cipher_suites.extend_from_copyable_slice(&u16::from(*cipher_suite).to_be_bytes())?;
        }
        cipher_suites
      }
      .as_slice(),
      &self.legacy_compression_methods,
    ])?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |sw| {
      if let Some(name) = self.tls_config.lease().server_name.as_ref() {
        ClientHelloExtension::new(ClientHelloExtensionTy::ServerName, name).encode(sw)?;
      }
      if let Some(max_fragment_length) = self.tls_config.lease().max_fragment_length {
        ClientHelloExtension::new(ClientHelloExtensionTy::MaxFragmentLength, max_fragment_length)
          .encode(sw)?;
      }
      ClientHelloExtension::new(
        ClientHelloExtensionTy::SupportedGroups,
        SupportedGroups { supported_groups: self.tls_config.lease().named_groups.clone() },
      )
      .encode(sw)?;
      ClientHelloExtension::new(
        ClientHelloExtensionTy::SignatureAlgorithms,
        SignatureAlgorithms {
          signature_schemes: ArrayVectorU8::from_iterator(
            self.tls_config.lease().signature_algorithms.iter().copied(),
          )?,
        },
      )
      .encode(sw)?;

      ClientHelloExtension::new(
        ClientHelloExtensionTy::SupportedVersions,
        &self.supported_versions,
      )
      .encode(sw)?;
      ClientHelloExtension::new(
        ClientHelloExtensionTy::PskKeyExchangeModes,
        &self.psk_key_exchange_modes,
      )
      .encode(sw)?;

      {
        let mut client_shares = ArrayVectorU8::<_, KEY_SHARES_LEN>::new();
        for (key_share, secret) in self.tls_config.lease().key_shares.iter().zip(&self.secrets) {
          let opaque = secret.public_key()?;
          client_shares.push((key_share.group, opaque))?;
        }
        ClientHelloExtension::new(
          ClientHelloExtensionTy::KeyShare,
          KeyShareClientHello {
            client_shares: ArrayVectorU8::from_iterator(
              client_shares
                .iter()
                .map(|(group, opaque)| KeyShareEntry { group: *group, opaque: &opaque }),
            )?,
          },
        )
        .encode(sw)?;
      }
      ClientHelloExtension::new(
        ClientHelloExtensionTy::SignatureAlgorithmsCert,
        SignatureAlgorithmsCert {
          supported_groups: self.tls_config.lease().signature_algorithms_cert.clone(),
        },
      )
      .encode(sw)?;

      if !self.tls_config.lease().offered_psks.offered_psks.is_empty() {
        ClientHelloExtension::new(
          ClientHelloExtensionTy::PreSharedKey,
          OfferedPsks { offered_psks: self.tls_config.lease().offered_psks.offered_psks.clone() },
        )
        .encode(sw)?;
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
