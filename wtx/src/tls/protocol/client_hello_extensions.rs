// Subset of https://datatracker.ietf.org/doc/html/rfc8446#section-4.2 for clients

use crate::{
  collection::{ArrayVector, ArrayVectorU8},
  de::{Decode, Encode},
  misc::{
    Lease, SuffixWriterMut,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  tls::{
    MAX_KEY_SHARES_LEN, MaxFragmentLength, TlsError,
    de::De,
    ephemeral_secret_key::EphemeralSecretKey,
    misc::u16_chunk,
    protocol::{
      client_hello_extension::ClientHelloExtension,
      client_hello_extension_ty::ClientHelloExtensionTy,
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
pub(crate) struct ClientHelloExtensions<S, TC> {
  pub(crate) psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  pub(crate) secrets: S,
  pub(crate) supported_versions: SupportedVersions,
  pub(crate) tls_config: TC,
}

impl<'any, S, TC> ClientHelloExtensions<S, TC>
where
  TC: Lease<TlsConfigInner<'any>>,
{
  pub(crate) fn new(secrets: S, tls_config: TC) -> Self {
    Self {
      psk_key_exchange_modes: Some(PskKeyExchangeModes::new(ArrayVectorU8::from_array([
        PskKeyExchangeMode::PskDheKe,
      ]))),
      secrets,
      supported_versions: SupportedVersions::new(ArrayVectorU8::from_array([
        ProtocolVersion::Tls13,
      ])),
      tls_config,
    }
  }

  pub(crate) fn tls_config(&self) -> &TlsConfigInner<'any> {
    self.tls_config.lease()
  }
}

impl<'de> Decode<'de, De> for ClientHelloExtensions<(), TlsConfigInner<'de>> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
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
          ClientHelloExtensionTy::PreSharedKey => {
            duplicated_error(!pre_shared_key.offered_psks.is_empty())?;
            pre_shared_key = ClientHelloExtension::<OfferedPsks<'_>>::decode(bytes)?.into_data();
          }
          ClientHelloExtensionTy::SupportedVersions => {
            duplicated_error(supported_versions_opt.is_some())?;
            supported_versions_opt =
              Some(ClientHelloExtension::<SupportedVersions>::decode(bytes)?.into_data());
          }
          ClientHelloExtensionTy::PskKeyExchangeModes => {
            duplicated_error(psk_key_exchange_modes.is_some())?;
            psk_key_exchange_modes =
              Some(ClientHelloExtension::<PskKeyExchangeModes>::decode(bytes)?.into_data());
          }
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
          ClientHelloExtensionTy::ApplicationLayerProtocolNegotiation
          | ClientHelloExtensionTy::CertificateAuthorities
          | ClientHelloExtensionTy::ClientCertificateType
          | ClientHelloExtensionTy::Cookie
          | ClientHelloExtensionTy::EarlyData
          | ClientHelloExtensionTy::Heartbeat
          | ClientHelloExtensionTy::Padding
          | ClientHelloExtensionTy::PostHandshakeAuth
          | ClientHelloExtensionTy::ServerCertificateType
          | ClientHelloExtensionTy::SignedCertificateTimestamp
          | ClientHelloExtensionTy::StatusRequest
          | ClientHelloExtensionTy::UseSrtp => {
            return Err(TlsError::UnsupportedExtension.into());
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
      psk_key_exchange_modes,
      secrets: (),
      supported_versions,
      tls_config: TlsConfigInner {
        root_ca: None,
        certificate: None,
        cipher_suites: ArrayVectorU8::new(),
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
  for ClientHelloExtensions<&'secrets ArrayVectorU8<ES, MAX_KEY_SHARES_LEN>, TC>
where
  ES: EphemeralSecretKey,
  TC: Lease<TlsConfigInner<'any>>,
{
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
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
        let mut client_shares = ArrayVectorU8::<_, MAX_KEY_SHARES_LEN>::new();
        for (key_share, secret) in self.tls_config.lease().key_shares.iter().zip(self.secrets) {
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
