// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.2

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  rng::CryptoRng,
  tls::{
    CipherSuiteTy, MAX_KEY_SHARES_LEN, MaxFragmentLength, TlsError,
    de::De,
    decode_wrapper::DecodeWrapper,
    encode_wrapper::EncodeWrapper,
    ephemeral_secret_key::EphemeralSecretKey,
    misc::{u8_chunk, u16_chunk, u16_list},
    protocol::{
      extension::Extension,
      extension_ty::ExtensionTy,
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
pub(crate) struct ClientHello<S, TC> {
  legacy_compression_methods: [u8; 2],
  legacy_session_id: ArrayVectorU8<u8, 32>,
  legacy_version: ProtocolVersion,
  psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  random: [u8; 32],
  secrets: S,
  supported_versions: SupportedVersions,
  tls_config: TC,
}

impl<'any, S, TC> ClientHello<S, TC>
where
  TC: Lease<TlsConfigInner<'any>>,
{
  pub(crate) fn new<RNG>(rng: &mut RNG, secrets: S, tls_config: TC) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self {
      legacy_compression_methods: [1, 0],
      legacy_session_id: ArrayVectorU8::from_array({
        let mut array = [0; 32];
        rng.fill_slice(&mut array[0..4]);
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
      supported_versions: SupportedVersions::new(ArrayVectorU8::from_array([
        ProtocolVersion::Tls13,
      ])),
      tls_config,
    })
  }

  pub(crate) fn legacy_session_id(&self) -> &ArrayVectorU8<u8, 32> {
    &self.legacy_session_id
  }

  pub(crate) fn tls_config(&self) -> &TlsConfigInner<'any> {
    self.tls_config.lease()
  }
}

impl<'de> Decode<'de, De> for ClientHello<(), TlsConfigInner<'de>> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32]>::decode(dw)?;
    let legacy_session_id =
      u8_chunk(dw, TlsError::InvalidLegacySessionId, |el| Ok(el.bytes()))?.try_into()?;
    let mut cipher_suites = ArrayVectorU8::new();
    let mut key_shares = ArrayVectorU8::new();
    let mut last_ty = None;
    let mut max_fragment_length = None;
    let mut named_groups = ArrayVectorU8::new();
    let mut pre_shared_key = OfferedPsks { offered_psks: ArrayVectorU8::new() };
    let mut psk_key_exchange_modes = None;
    let mut server_name = None;
    let mut signature_algorithms = ArrayVectorU8::new();
    let mut signature_algorithms_cert = ArrayVectorU8::new();
    let mut supported_versions_opt = None;
    u16_list(&mut cipher_suites, dw, TlsError::InvalidCipherSuite)?;
    let legacy_compression_methods = <[u8; 2]>::decode(dw)?;
    u16_chunk(dw, TlsError::InvalidClientHelloLength, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *local_dw;
          ExtensionTy::decode(tmp_bytes)?
        };
        last_ty = Some(extension_ty);
        match extension_ty {
          ExtensionTy::ServerName => {
            duplicated_error(server_name.is_some())?;
            server_name = Some(Extension::<ServerNameList>::decode(local_dw)?.into_data());
          }
          ExtensionTy::MaxFragmentLength => {
            duplicated_error(max_fragment_length.is_some())?;
            max_fragment_length =
              Some(Extension::<MaxFragmentLength>::decode(local_dw)?.into_data());
          }
          ExtensionTy::SupportedGroups => {
            duplicated_error(!named_groups.is_empty())?;
            named_groups =
              Extension::<SupportedGroups>::decode(local_dw)?.into_data().supported_groups;
          }
          ExtensionTy::SignatureAlgorithms => {
            duplicated_error(!signature_algorithms.is_empty())?;
            signature_algorithms =
              Extension::<SignatureAlgorithms>::decode(local_dw)?.into_data().signature_schemes;
          }
          ExtensionTy::PreSharedKey => {
            duplicated_error(!pre_shared_key.offered_psks.is_empty())?;
            pre_shared_key = Extension::<OfferedPsks<'_>>::decode(local_dw)?.into_data();
          }
          ExtensionTy::SupportedVersions => {
            duplicated_error(supported_versions_opt.is_some())?;
            supported_versions_opt =
              Some(Extension::<SupportedVersions>::decode(local_dw)?.into_data());
          }
          ExtensionTy::PskKeyExchangeModes => {
            duplicated_error(psk_key_exchange_modes.is_some())?;
            psk_key_exchange_modes =
              Some(Extension::<PskKeyExchangeModes>::decode(local_dw)?.into_data());
          }
          ExtensionTy::SignatureAlgorithmsCert => {
            duplicated_error(!signature_algorithms_cert.is_empty())?;
            signature_algorithms_cert =
              Extension::<SignatureAlgorithmsCert>::decode(local_dw)?.into_data().supported_groups;
          }
          ExtensionTy::KeyShare => {
            duplicated_error(!key_shares.is_empty())?;
            key_shares =
              Extension::<KeyShareClientHello<'_>>::decode(local_dw)?.into_data().client_shares;
          }
          ExtensionTy::OidFilters => {
            return Err(TlsError::MismatchedExtension.into());
          }
          _ => {
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

impl<'any, 'secrets, ES, TC> Encode<De>
  for ClientHello<&'secrets ArrayVectorU8<ES, MAX_KEY_SHARES_LEN>, TC>
where
  ES: EphemeralSecretKey,
  TC: Lease<TlsConfigInner<'any>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slices([
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
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      if let Some(name) = self.tls_config.lease().server_name.as_ref() {
        Extension::new(ExtensionTy::ServerName, name).encode(local_ew)?;
      }
      if let Some(max_fragment_length) = self.tls_config.lease().max_fragment_length {
        Extension::new(ExtensionTy::MaxFragmentLength, max_fragment_length).encode(local_ew)?;
      }
      Extension::new(
        ExtensionTy::SupportedGroups,
        SupportedGroups { supported_groups: self.tls_config.lease().named_groups.clone() },
      )
      .encode(local_ew)?;
      Extension::new(
        ExtensionTy::SignatureAlgorithms,
        SignatureAlgorithms {
          signature_schemes: ArrayVectorU8::from_iterator(
            self.tls_config.lease().signature_algorithms.iter().copied(),
          )?,
        },
      )
      .encode(local_ew)?;
      Extension::new(ExtensionTy::SupportedVersions, &self.supported_versions).encode(local_ew)?;
      Extension::new(ExtensionTy::PskKeyExchangeModes, &self.psk_key_exchange_modes)
        .encode(local_ew)?;
      {
        let mut client_shares = ArrayVectorU8::<_, MAX_KEY_SHARES_LEN>::new();
        for (key_share, secret) in self.tls_config.lease().key_shares.iter().zip(self.secrets) {
          let opaque = secret.public_key()?;
          client_shares.push((key_share.group, opaque))?;
        }
        Extension::new(
          ExtensionTy::KeyShare,
          KeyShareClientHello {
            client_shares: ArrayVectorU8::from_iterator(
              client_shares
                .iter()
                .map(|(group, opaque)| KeyShareEntry { group: *group, opaque: &opaque }),
            )?,
          },
        )
        .encode(local_ew)?;
      }
      Extension::new(
        ExtensionTy::SignatureAlgorithmsCert,
        SignatureAlgorithmsCert {
          supported_groups: self.tls_config.lease().signature_algorithms_cert.clone(),
        },
      )
      .encode(local_ew)?;
      if !self.tls_config.lease().offered_psks.offered_psks.is_empty() {
        Extension::new(
          ExtensionTy::PreSharedKey,
          OfferedPsks { offered_psks: self.tls_config.lease().offered_psks.offered_psks.clone() },
        )
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
