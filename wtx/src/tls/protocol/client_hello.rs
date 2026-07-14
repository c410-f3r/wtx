// https://datatracker.ietf.org/doc/html/rfc9846#section-4.1.2

use crate::{
  calendar::DateTime,
  codec::{Decode, Encode},
  collections::{ArrayVectorCopy, ArrayVectorU8, Vector},
  misc::{
    Lease, SingleTypeStorage,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  rng::CryptoRng,
  tls::{
    CipherSuite, MaxFragmentLength, NamedGroup, TlsConfig, TlsError, TlsMode,
    de::De,
    misc::{u8_chunk, u16_chunk},
    protocol::{
      alpn::Alpn, extension::Extension, extension_ty::ExtensionTy,
      key_share_client_hello::KeyShareClientHello, key_share_entry::KeyShareEntry,
      named_group::NamedGroupAgreement, protocol_version::ProtocolVersion,
      protocol_versions::SupportedVersionsClient, server_name_list::ServerNameList,
      signature_algorithms::SignatureAlgorithms,
      signature_algorithms_cert::SignatureAlgorithmsCert, supported_groups::SupportedGroups,
    },
    tls_config::TlsConfigInner,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
  x509::CvPolicy,
};

#[derive(Debug)]
pub(crate) struct ClientHello<G, TC> {
  generic: G,
  legacy_session_id: ArrayVectorCopy<u8, 32>,
  legacy_version: ProtocolVersion,
  random: [u8; 32],
  supported_versions: SupportedVersionsClient,
  tls_config: TC,
}

impl<G, TC> ClientHello<G, TC> {
  pub(crate) fn new<RNG>(generic: G, rng: &mut RNG, tls_config: TC) -> Self
  where
    RNG: CryptoRng,
  {
    Self {
      generic,
      legacy_session_id: ArrayVectorCopy::from_array({
        let mut array = [0; 32];
        rng.fill_slice(&mut array);
        array
      }),
      legacy_version: ProtocolVersion::Tls12,
      random: {
        let mut array = [0u8; 32];
        rng.fill_slice(&mut array);
        array
      },
      supported_versions: SupportedVersionsClient::new(ArrayVectorCopy::from_array([
        ProtocolVersion::Tls13,
      ])),
      tls_config,
    }
  }

  pub(crate) fn generic(&self) -> &G {
    &self.generic
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

impl<'de, TM> Decode<'de, De>
  for ClientHello<KeyShareClientHello<&'de [u8]>, TlsConfigInner<&'de [u8], TM>>
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
    let mut key_shares_opt = None;
    let mut last_ty = None;
    let mut max_fragment_length = None;
    let mut server_name = None;
    let mut signature_algorithms_opt = None;
    let mut signature_algorithms_cert = None;
    let mut supported_groups_opt = None;
    let mut supported_versions_opt = None;
    {
      let bytes = u16_chunk(dw, TlsError::InvalidCipherSuite, |el| Ok(el.bytes()))?;
      for [b0, b1] in bytes.as_chunks::<2>().0 {
        if let Ok(elem) = CipherSuite::try_from(u16::from_be_bytes([*b0, *b1])) {
          cipher_suites.push(elem)?;
        }
      }
    }
    let _legacy_compression_methods @ [1, 0] = <[u8; 2] as Decode<'de, De>>::decode(dw)? else {
      return Err(TlsError::InvalidLegacyCompressionMethod.into());
    };
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let Ok(extension_ty) = ExtensionTy::decode(local_dw) else {
          u16_chunk(local_dw, err, |_bytes| Ok(()))?;
          continue;
        };
        last_ty = Some(extension_ty);
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(
            &mut alpn,
            local_local_dw,
            extension_ty,
            &mut key_shares_opt,
            &mut max_fragment_length,
            &mut server_name,
            &mut signature_algorithms_opt,
            &mut signature_algorithms_cert,
            &mut supported_groups_opt,
            &mut supported_versions_opt,
          )
        })?;
      }
      Ok(())
    })?;
    let Some(supported_versions) = supported_versions_opt else {
      return Err(TlsError::MissingSupportedVersions.into());
    };
    let Some(signature_algorithms) = signature_algorithms_opt else {
      return Err(TlsError::MissingSignatureAlgorithms.into());
    };
    let Some(supported_groups) = supported_groups_opt else {
      return Err(TlsError::MissingSupportedGroups.into());
    };
    let Some(key_shares) = key_shares_opt else {
      return Err(TlsError::MissingKeyShares.into());
    };
    Ok(Self {
      generic: key_shares,
      legacy_session_id,
      legacy_version,
      random,
      supported_versions,
      tls_config: TlsConfigInner {
        alpn,
        cipher_suites,
        cv_policy: CvPolicy::new(DateTime::default()),
        max_fragment_length,
        supported_groups,
        public_key: Vector::new(),
        secret_key: &[],
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
  for ClientHello<&ArrayVectorU8<NamedGroupAgreement, { NamedGroup::len() }>, TC>
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
        let mut cipher_suites = ArrayVectorCopy::<_, { 2 * CipherSuite::ALL.len() }>::new();
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
      {
        let mut client_shares = ArrayVectorU8::new();
        for secret in self.generic {
          client_shares
            .push(KeyShareEntry { group: secret.named_group(), opaque: secret.public_key()? })?;
        }
        Extension::new(ExtensionTy::KeyShare, KeyShareClientHello { client_shares })
          .encode(local_ew)?;
      }
      if let Some(max_fragment_length) = self.tls_config.lease().inner.max_fragment_length {
        Extension::new(ExtensionTy::MaxFragmentLength, max_fragment_length).encode(local_ew)?;
      }
      if let Some(name) = self.tls_config.lease().inner.server_name.as_ref() {
        Extension::new(ExtensionTy::ServerName, name).encode(local_ew)?;
      }
      Extension::new(
        ExtensionTy::SignatureAlgorithms,
        &self.tls_config.lease().inner.signature_algorithms,
      )
      .encode(local_ew)?;
      Extension::new(
        ExtensionTy::SignatureAlgorithmsCert,
        &self.tls_config.lease().inner.signature_algorithms_cert,
      )
      .encode(local_ew)?;
      Extension::new(ExtensionTy::SupportedGroups, &self.tls_config.lease().inner.supported_groups)
        .encode(local_ew)?;
      Extension::new(ExtensionTy::SupportedVersions, &self.supported_versions).encode(local_ew)?;
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
  dw: &mut TlsDecodeWrapper<'de>,
  extension_ty: ExtensionTy,
  key_shares: &mut Option<KeyShareClientHello<&'de [u8]>>,
  max_fragment_length: &mut Option<MaxFragmentLength>,
  server_name: &mut Option<ServerNameList>,
  signature_algorithms: &mut Option<SignatureAlgorithms>,
  signature_algorithms_cert: &mut Option<SignatureAlgorithmsCert>,
  supported_groups: &mut Option<SupportedGroups>,
  supported_versions_opt: &mut Option<SupportedVersionsClient>,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::ApplicationLayerProtocolNegotiation => {
      duplicated_error(alpn.is_some())?;
      *alpn = Some(Alpn::decode(dw)?);
    }
    ExtensionTy::MaxFragmentLength => {
      duplicated_error(max_fragment_length.is_some())?;
      *max_fragment_length = Some(MaxFragmentLength::decode(dw)?);
    }
    ExtensionTy::KeyShare => {
      duplicated_error(key_shares.is_some())?;
      *key_shares = Some(KeyShareClientHello::<&[u8]>::decode(dw)?);
    }
    ExtensionTy::ServerName => {
      duplicated_error(server_name.is_some())?;
      *server_name = Some(ServerNameList::decode(dw)?);
    }
    ExtensionTy::SignatureAlgorithms => {
      duplicated_error(signature_algorithms.is_some())?;
      *signature_algorithms = Some(SignatureAlgorithms::decode(dw)?);
    }
    ExtensionTy::SignatureAlgorithmsCert => {
      duplicated_error(signature_algorithms_cert.is_some())?;
      *signature_algorithms_cert = Some(SignatureAlgorithmsCert::decode(dw)?);
    }
    ExtensionTy::SupportedGroups => {
      duplicated_error(supported_groups.is_some())?;
      *supported_groups = Some(SupportedGroups::decode(dw)?);
    }
    ExtensionTy::SupportedVersions => {
      duplicated_error(supported_versions_opt.is_some())?;
      *supported_versions_opt = Some(SupportedVersionsClient::decode(dw)?);
    }
    ExtensionTy::CertificateAuthorities
    | ExtensionTy::ClientCertificateType
    | ExtensionTy::Cookie
    | ExtensionTy::EarlyData
    | ExtensionTy::Heartbeat
    | ExtensionTy::Padding
    | ExtensionTy::PostHandshakeAuth
    | ExtensionTy::PreSharedKey
    | ExtensionTy::PskKeyExchangeModes
    | ExtensionTy::ServerCertificateType
    | ExtensionTy::SignedCertificateTimestamp
    | ExtensionTy::StatusRequest
    | ExtensionTy::UseSrtp => {}
    ExtensionTy::OidFilters => {
      return Err(TlsError::MismatchedExtension.into());
    }
  }
  Ok(())
}
