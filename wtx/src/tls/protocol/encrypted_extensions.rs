// https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.1

use crate::{
  codec::{Decode, Encode},
  collections::{ArrayVectorCopy, ArrayVectorU8},
  rng::CryptoRng,
  tls::{
    CipherSuite, MaxFragmentLength, TlsCertificateTy, TlsError,
    de::De,
    misc::{duplicated_error, u16_chunk},
    protocol::{
      alpn::Alpn, cert_type::CertType, extension::Extension, extension_ty::ExtensionTy,
      key_share_entry::KeyShareEntry, protocol_version::ProtocolVersion,
      protocol_versions::SupportedVersions, server_name_list::ServerNameList,
      supported_groups::SupportedGroups,
    },
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct EncryptedExtensions<'any> {
  alpn: Alpn,
  cipher_suite: CipherSuite,
  client_cert_type: Option<TlsCertificateTy>,
  key_share: KeyShareEntry<&'any [u8]>,
  legacy_compression_method: u8,
  legacy_session_id_echo: ArrayVectorU8<u8, 32>,
  legacy_version: ProtocolVersion,
  random: [u8; 32],
  selected_identity: Option<u16>,
  server_cert_type: Option<TlsCertificateTy>,
  supported_versions: SupportedVersions,
}

impl<'any> EncryptedExtensions<'any> {
  pub(crate) fn new<RNG>(
    alpn: Alpn,
    cipher_suite: CipherSuite,
    client_cert_type: Option<TlsCertificateTy>,
    key_share: KeyShareEntry<&'any [u8]>,
    legacy_session_id_echo: ArrayVectorU8<u8, 32>,
    rng: &mut RNG,
    selected_identity: Option<u16>,
    server_cert_type: Option<TlsCertificateTy>,
  ) -> Self
  where
    RNG: CryptoRng,
  {
    let mut random = [0u8; 32];
    rng.fill_slice(&mut random);
    Self {
      alpn,
      cipher_suite,
      client_cert_type,
      key_share,
      legacy_compression_method: 0,
      legacy_session_id_echo,
      legacy_version: ProtocolVersion::Tls12,
      random,
      selected_identity,
      server_cert_type,
      supported_versions: SupportedVersions::new(ArrayVectorCopy::from_array([
        ProtocolVersion::Tls13,
      ])),
    }
  }
}

impl<'de> Decode<'de, De> for EncryptedExtensions<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32] as Decode<'de, De>>::decode(dw)?;
    let legacy_session_id_echo =
      u16_chunk(dw, TlsError::InvalidLegacySessionIdEcho, |el| Ok(el.bytes()))?.try_into()?;
    let cipher_suite = CipherSuite::decode(dw)?;
    let legacy_compression_method = <u8 as Decode<'de, De>>::decode(dw)?;
    let mut alpn = Alpn { protocol_name_list: ArrayVectorCopy::new() };
    let mut client_cert_type = None;
    let mut key_share_opt = None;
    let mut max_fragment_length: Option<MaxFragmentLength> = None;
    let mut named_groups = ArrayVectorCopy::new();
    let mut selected_identity = None;
    let mut server_cert_type = None;
    let mut server_name = None;
    let mut supported_versions_opt: Option<SupportedVersions> = None;
    u16_chunk(dw, TlsError::InvalidClientHelloLength, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *local_dw;
          ExtensionTy::decode(tmp_bytes)?
        };
        match extension_ty {
          ExtensionTy::ApplicationLayerProtocolNegotiation => {
            duplicated_error(!alpn.protocol_name_list.is_empty())?;
            alpn = Alpn::decode(local_dw)?;
          }
          ExtensionTy::ClientCertificateType => {
            duplicated_error(client_cert_type.is_some())?;
            client_cert_type = Some(Extension::<CertType>::decode(local_dw)?.into_data().0);
          }
          ExtensionTy::KeyShare => {
            duplicated_error(key_share_opt.is_some())?;
            key_share_opt = Some(KeyShareEntry::decode(local_dw)?);
          }
          ExtensionTy::MaxFragmentLength => {
            duplicated_error(max_fragment_length.is_some())?;
            max_fragment_length = Some(Extension::<_>::decode(local_dw)?.into_data());
          }
          ExtensionTy::PreSharedKey => {
            duplicated_error(selected_identity.is_some())?;
            selected_identity = Some(Extension::<u16>::decode(local_dw)?.into_data());
          }
          ExtensionTy::ServerCertificateType => {
            duplicated_error(server_cert_type.is_some())?;
            server_cert_type = Some(Extension::<CertType>::decode(local_dw)?.into_data().0);
          }
          ExtensionTy::ServerName => {
            duplicated_error(server_name.is_some())?;
            server_name =
              Some(Extension::<ServerNameList<&'de [u8]>>::decode(local_dw)?.into_data());
          }
          ExtensionTy::SupportedGroups => {
            duplicated_error(!named_groups.is_empty())?;
            named_groups =
              Extension::<SupportedGroups>::decode(local_dw)?.into_data().supported_groups;
          }
          ExtensionTy::SupportedVersions => {
            duplicated_error(supported_versions_opt.is_some())?;
            supported_versions_opt = Some(Extension::<_>::decode(local_dw)?.into_data());
          }
          ExtensionTy::EarlyData
          | ExtensionTy::Heartbeat
          | ExtensionTy::StatusRequest
          | ExtensionTy::UseSrtp => {
            return Err(TlsError::UnsupportedExtension.into());
          }
          ExtensionTy::CertificateAuthorities
          | ExtensionTy::Cookie
          | ExtensionTy::OidFilters
          | ExtensionTy::Padding
          | ExtensionTy::PostHandshakeAuth
          | ExtensionTy::PskKeyExchangeModes
          | ExtensionTy::SignatureAlgorithms
          | ExtensionTy::SignatureAlgorithmsCert
          | ExtensionTy::SignedCertificateTimestamp => {
            return Err(TlsError::MismatchedExtension.into());
          }
        }
      }
      Ok(())
    })?;
    let Some(supported_versions) = supported_versions_opt else {
      return Err(TlsError::MissingSupportedVersions.into());
    };
    let [ProtocolVersion::Tls13] = supported_versions.versions.as_slice() else {
      return Err(TlsError::UnsupportedTlsVersion.into());
    };
    Ok(Self {
      alpn,
      cipher_suite,
      client_cert_type,
      key_share: key_share_opt.ok_or(TlsError::MissingKeyShares)?,
      legacy_compression_method,
      legacy_session_id_echo,
      legacy_version,
      random,
      selected_identity,
      server_cert_type,
      supported_versions,
    })
  }
}

impl Encode<De> for EncryptedExtensions<'_> {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    let Self {
      alpn,
      cipher_suite,
      client_cert_type,
      key_share,
      legacy_compression_method,
      legacy_session_id_echo,
      legacy_version,
      random,
      selected_identity,
      server_cert_type,
      supported_versions,
    } = self;
    legacy_version.encode(ew)?;
    let _ = ew.buffer().extend_from_copyable_slices([random.as_slice(), legacy_session_id_echo])?;
    cipher_suite.encode(ew)?;
    ew.buffer().push(*legacy_compression_method)?;
    Extension::new(ExtensionTy::ApplicationLayerProtocolNegotiation, alpn).encode(ew)?;
    if let Some(el) = client_cert_type {
      Extension::new(ExtensionTy::ClientCertificateType, CertType(*el)).encode(ew)?;
    }
    Extension::new(ExtensionTy::KeyShare, key_share).encode(ew)?;
    Extension::new(ExtensionTy::PreSharedKey, selected_identity).encode(ew)?;
    if let Some(el) = server_cert_type {
      Extension::new(ExtensionTy::ServerCertificateType, CertType(*el)).encode(ew)?;
    }
    Extension::new(ExtensionTy::SupportedVersions, supported_versions).encode(ew)?;
    Ok(())
  }
}
