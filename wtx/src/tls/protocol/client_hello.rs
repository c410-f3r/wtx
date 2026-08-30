// https://datatracker.ietf.org/doc/html/rfc9846#section-4.1.2

use crate::{
  calendar::DateTime,
  codec::{Decode, Encode},
  collections::{ArrayVectorCopy, ArrayVectorU8, ShortBoxSliceU8, SingleTypeStorage},
  misc::{
    Lease,
    counter_writer::{CounterWriterBytesTy, u16_write},
  },
  rng::CryptoRng,
  tls::{
    AlertDescription, CipherSuite, MaxFragmentLength, NamedGroup, PublicKeys, TlsConfig, TlsError,
    misc::{decode_extension_ty, u8_chunk, u16_chunk},
    protocol::{
      alpn::Alpn, certificate_authorities::CertificateAuthorities, extension::Extension,
      extension_ty::ExtensionTy, key_share_client_hello::KeyShareClientHello,
      key_share_entry::KeyShareEntry, named_group::NamedGroupAgreement,
      protocol_version::ProtocolVersion, protocol_versions::SupportedVersionsClient,
      psk_key_exchange_modes::PskKeyExchangeModes, server_name_list::ServerNameList,
      signature_algorithms::SignatureAlgorithms,
      signature_algorithms_cert::SignatureAlgorithmsCert, supported_groups::SupportedGroups,
    },
    tls_cc::TlsCc,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
    tls_config::TlsConfigInner,
  },
  x509::CvPolicy,
};

#[derive(Debug)]
pub(crate) struct ClientHello<G, TCG> {
  generic: G,
  legacy_session_id: ArrayVectorCopy<u8, 32>,
  random: [u8; 32],
  supported_versions: SupportedVersionsClient,
  tls_config: TCG,
}

impl<G, TCG> ClientHello<G, TCG> {
  pub(crate) fn new<RNG>(generic: G, rng: &mut RNG, tls_config: TCG) -> Self
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

  pub(crate) const fn generic(&self) -> &G {
    &self.generic
  }

  pub(crate) const fn legacy_session_id(&self) -> &ArrayVectorCopy<u8, 32> {
    &self.legacy_session_id
  }

  pub(crate) const fn supported_versions(&self) -> &SupportedVersionsClient {
    &self.supported_versions
  }

  pub(crate) const fn tls_config(&self) -> &TCG {
    &self.tls_config
  }
}

impl<'de> Decode<'de, TlsCc>
  for ClientHello<KeyShareClientHello<&'de [u8]>, TlsConfigInner<&'de [u8], ()>>
{
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidClientHelloLength;
    let _legacy_version = <[u8; 2] as Decode<'_, TlsCc>>::decode(dw)?;
    let random = <[u8; 32] as Decode<'de, TlsCc>>::decode(dw)?;
    let legacy_session_id = u8_chunk(dw, err, |el| Ok(el.bytes()))?.try_into().map_err(|_err| {
      crate::Error::TlsErrorReply(TlsError::InvalidLegacySessionId, AlertDescription::DecodeError)
    })?;
    let mut cipher_suites = ArrayVectorCopy::new();
    let mut extensions = Extensions::default();
    {
      let bytes = u16_chunk(dw, TlsError::InvalidCipherSuite, |el| Ok(el.bytes()))?;
      for [b0, b1] in bytes.as_chunks::<2>().0 {
        if let Ok(elem) = CipherSuite::try_from(u16::from_be_bytes([*b0, *b1])) {
          cipher_suites.push(elem)?;
        }
      }
    }
    let _legacy_compression_methods @ Ok([1, 0]) = <[u8; 2] as Decode<'de, TlsCc>>::decode(dw)
    else {
      return Err(crate::Error::TlsErrorReply(
        TlsError::InvalidLegacyCompressionMethods,
        AlertDescription::IllegalParameter,
      ));
    };
    u16_chunk(dw, err, |local_dw| {
      let mut seen_unknowns = ArrayVectorCopy::new();
      while !local_dw.bytes().is_empty() {
        let Some(extension_ty) = decode_extension_ty(local_dw, err, &mut seen_unknowns)? else {
          continue;
        };
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(local_local_dw, extension_ty, &mut extensions)
        })?;
      }
      Ok(())
    })?;
    let Some(supported_versions) = extensions.supported_versions else {
      return Err(TlsError::MissingSupportedVersions.into());
    };
    let Some(signature_algorithms) = extensions.signature_algorithms else {
      return Err(TlsError::MissingSignatureAlgorithms.into());
    };
    let Some(supported_groups) = extensions.supported_groups else {
      return Err(crate::Error::TlsErrorReply(
        TlsError::MissingSupportedGroups,
        AlertDescription::MissingExtension,
      ));
    };
    let Some(key_shares) = extensions.key_shares else {
      return Err(crate::Error::TlsErrorReply(
        TlsError::MissingKeyShares,
        AlertDescription::MissingExtension,
      ));
    };
    Ok(Self {
      generic: key_shares,
      legacy_session_id,
      random,
      supported_versions,
      tls_config: TlsConfigInner {
        alpn: extensions.alpn,
        cipher_suites,
        ctx: (),
        cv_policy: CvPolicy::new(DateTime::default()),
        max_fragment_length: extensions.max_fragment_length,
        max_fragment_length_send: None,
        supported_groups,
        public_keys: PublicKeys::default(),
        server_name: extensions.server_name,
        signature_algorithms,
        signature_algorithms_cert: extensions.signature_algorithms_cert,
        trust_anchors: ShortBoxSliceU8::default(),
        unique_signature_algorithms: false,
      },
    })
  }
}

impl<TCG, TCX> Encode<TlsCc>
  for ClientHello<&ArrayVectorU8<NamedGroupAgreement, { NamedGroup::len() }>, TCG>
where
  TCG: Lease<TlsConfig<TCX>> + SingleTypeStorage<Item = TCX>,
{
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let _ = ew.buffer().extend_from_copyable_slices([
      u16::from(ProtocolVersion::Tls12).to_be_bytes().as_slice(),
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
      {
        let mut client_shares = ArrayVectorU8::new();
        for secret in self.generic {
          client_shares.push(KeyShareEntry::new(secret.named_group(), secret.public_key()?))?;
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
      Extension::new(ExtensionTy::PskKeyExchangeModes, PskKeyExchangeModes {}).encode(local_ew)?;
      crate::Result::Ok(())
    })?;
    Ok(())
  }
}

const fn duplicated_error(is_some: bool) -> crate::Result<()> {
  if is_some {
    return Err(crate::Error::TlsErrorReply(
      TlsError::DuplicatedClientHelloParameters,
      AlertDescription::DecodeError,
    ));
  }
  Ok(())
}

#[expect(clippy::too_many_lines, reason = "up to the specification")]
#[inline]
fn manage_extension<'de>(
  dw: &mut TlsDecodeWrapper<'de>,
  extension_ty: ExtensionTy,
  extensions: &mut Extensions<'de>,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::ApplicationLayerProtocolNegotiation => {
      duplicated_error(extensions.alpn.is_some())?;
      extensions.alpn = Some(Alpn::decode(dw)?);
    }
    ExtensionTy::CertificateAuthorities => {
      duplicated_error(extensions.certificate_authorities)?;
      extensions.certificate_authorities = true;
      let certificate_authorities = CertificateAuthorities::<&[u8]>::decode(dw)?;
      if !dw.bytes().is_empty() {
        return Err(crate::Error::TlsErrorReply(
          TlsError::TrailingDataInExtension,
          AlertDescription::DecodeError,
        ));
      }
      if certificate_authorities.authorities.is_empty() {
        return Err(crate::Error::TlsErrorReply(
          TlsError::EmptyCertificateAuthorities,
          AlertDescription::DecodeError,
        ));
      }
    }
    ExtensionTy::ClientCertificateType => {
      duplicated_error(extensions.client_certificate_type)?;
      extensions.client_certificate_type = true;
    }
    ExtensionTy::Cookie => {
      duplicated_error(extensions.cookie)?;
      extensions.cookie = true;
    }
    ExtensionTy::EarlyData => {
      duplicated_error(extensions.early_data)?;
      extensions.early_data = true;
    }
    ExtensionTy::Heartbeat => {
      duplicated_error(extensions.heartbeat)?;
      extensions.heartbeat = true;
    }
    ExtensionTy::MaxFragmentLength => {
      duplicated_error(extensions.max_fragment_length.is_some())?;
      extensions.max_fragment_length = Some(MaxFragmentLength::decode(dw)?);
    }
    ExtensionTy::Padding => {
      duplicated_error(extensions.padding)?;
      extensions.padding = true;
    }
    ExtensionTy::PostHandshakeAuth => {
      duplicated_error(extensions.post_handshake_auth)?;
      extensions.post_handshake_auth = true;
    }
    ExtensionTy::PreSharedKey => {
      duplicated_error(extensions.pre_shared_key)?;
      extensions.pre_shared_key = true;
    }
    ExtensionTy::PskKeyExchangeModes => {
      duplicated_error(extensions.psk_key_exchange_modes.is_some())?;
      extensions.psk_key_exchange_modes = Some(PskKeyExchangeModes {});
    }
    ExtensionTy::KeyShare => {
      duplicated_error(extensions.key_shares.is_some())?;
      extensions.key_shares = Some(KeyShareClientHello::<&[u8]>::decode(dw)?);
      if !dw.bytes().is_empty() {
        return Err(TlsError::TrailingDataInExtension.into());
      }
    }
    ExtensionTy::ServerCertificateType => {
      duplicated_error(extensions.server_certificate_type)?;
      extensions.server_certificate_type = true;
    }
    ExtensionTy::ServerName => {
      duplicated_error(extensions.server_name.is_some())?;
      extensions.server_name = Some(ServerNameList::decode(dw)?);
      if !dw.bytes().is_empty() {
        return Err(crate::Error::TlsErrorReply(
          TlsError::TrailingDataInExtension,
          AlertDescription::DecodeError,
        ));
      }
    }
    ExtensionTy::SignedCertificateTimestamp => {
      duplicated_error(extensions.signed_certificate_timestamp)?;
      extensions.signed_certificate_timestamp = true;
    }
    ExtensionTy::SignatureAlgorithms => {
      duplicated_error(extensions.signature_algorithms.is_some())?;
      extensions.signature_algorithms = Some(SignatureAlgorithms::decode(dw)?);
    }
    ExtensionTy::SignatureAlgorithmsCert => {
      duplicated_error(extensions.signature_algorithms_cert.is_some())?;
      extensions.signature_algorithms_cert = Some(SignatureAlgorithmsCert::decode(dw)?);
    }
    ExtensionTy::StatusRequest => {
      duplicated_error(extensions.status_request)?;
      extensions.status_request = true;
    }
    ExtensionTy::SupportedGroups => {
      duplicated_error(extensions.supported_groups.is_some())?;
      extensions.supported_groups = Some(SupportedGroups::decode(dw)?);
    }
    ExtensionTy::SupportedVersions => {
      duplicated_error(extensions.supported_versions.is_some())?;
      extensions.supported_versions = Some(SupportedVersionsClient::decode(dw)?);
    }
    ExtensionTy::UseSrtp => {
      duplicated_error(extensions.use_srtp)?;
      extensions.use_srtp = true;
    }
    ExtensionTy::OidFilters => {
      return Err(crate::Error::TlsErrorReply(
        TlsError::MismatchedExtension,
        AlertDescription::BadRecordMac,
      ));
    }
  }
  Ok(())
}

#[derive(Debug, Default)]
struct Extensions<'de> {
  alpn: Option<Alpn>,
  certificate_authorities: bool,
  client_certificate_type: bool,
  cookie: bool,
  early_data: bool,
  heartbeat: bool,
  key_shares: Option<KeyShareClientHello<&'de [u8]>>,
  max_fragment_length: Option<MaxFragmentLength>,
  padding: bool,
  post_handshake_auth: bool,
  pre_shared_key: bool,
  psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  server_certificate_type: bool,
  server_name: Option<ServerNameList>,
  signature_algorithms_cert: Option<SignatureAlgorithmsCert>,
  signature_algorithms: Option<SignatureAlgorithms>,
  signed_certificate_timestamp: bool,
  status_request: bool,
  supported_groups: Option<SupportedGroups>,
  supported_versions: Option<SupportedVersionsClient>,
  use_srtp: bool,
}
