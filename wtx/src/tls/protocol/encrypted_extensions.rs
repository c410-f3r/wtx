// https://datatracker.ietf.org/doc/html/rfc8446#section-4.3.1

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  rng::CryptoRng,
  tls::{
    CipherSuiteTy, MaxFragmentLength, TlsError,
    de::De,
    decode_wrapper::DecodeWrapper,
    encode_wrapper::EncodeWrapper,
    misc::{duplicated_error, u16_chunk},
    protocol::{
      extension::Extension, extension_ty::ExtensionTy, key_share_entry::KeyShareEntry,
      protocol_version::ProtocolVersion, protocol_versions::SupportedVersions,
      server_name_list::ServerNameList, supported_groups::SupportedGroups,
    },
  },
};

#[derive(Debug)]
pub(crate) struct EncryptedExtensions<'any> {
  cipher_suite: CipherSuiteTy,
  key_share: KeyShareEntry<'any>,
  legacy_compression_method: u8,
  legacy_session_id_echo: ArrayVectorU8<u8, 32>,
  legacy_version: ProtocolVersion,
  random: [u8; 32],
  selected_identity: Option<u16>,
  supported_versions: SupportedVersions,
}

impl<'any> EncryptedExtensions<'any> {
  pub(crate) fn new<RNG>(
    cipher_suite: CipherSuiteTy,
    key_share: KeyShareEntry<'any>,
    legacy_session_id_echo: ArrayVectorU8<u8, 32>,
    rng: &mut RNG,
    selected_identity: Option<u16>,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut random = [0u8; 32];
    rng.fill_slice(&mut random);
    Ok(Self {
      cipher_suite,
      key_share,
      legacy_compression_method: 0,
      legacy_session_id_echo,
      legacy_version: ProtocolVersion::Tls12,
      random,
      selected_identity,
      supported_versions: SupportedVersions::new(ArrayVectorU8::from_array([
        ProtocolVersion::Tls13,
      ])),
    })
  }

  pub(crate) fn cipher_suite(&self) -> CipherSuiteTy {
    self.cipher_suite
  }

  pub(crate) fn key_share(&self) -> &KeyShareEntry<'any> {
    &self.key_share
  }
}

impl<'de> Decode<'de, De> for EncryptedExtensions<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32]>::decode(dw)?;
    let legacy_session_id_echo =
      u16_chunk(dw, TlsError::InvalidLegacySessionIdEcho, |el| Ok(el.bytes()))?.try_into()?;
    let cipher_suite = CipherSuiteTy::decode(dw)?;
    let legacy_compression_method = <u8 as Decode<'de, De>>::decode(dw)?;
    let mut key_share_opt = None;
    let mut named_groups = ArrayVectorU8::new();
    let mut max_fragment_length = None;
    let mut selected_identity = None;
    let mut server_name = None;
    let mut supported_versions_opt = None;
    u16_chunk(dw, TlsError::InvalidClientHelloLength, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *local_dw;
          ExtensionTy::decode(tmp_bytes)?
        };
        match extension_ty {
          ExtensionTy::KeyShare => {
            duplicated_error(key_share_opt.is_some())?;
            key_share_opt = Some(KeyShareEntry::decode(local_dw)?);
          }
          ExtensionTy::MaxFragmentLength => {
            duplicated_error(max_fragment_length.is_some())?;
            max_fragment_length =
              Some(Extension::<MaxFragmentLength>::decode(local_dw)?.into_data());
          }
          ExtensionTy::PreSharedKey => {
            duplicated_error(selected_identity.is_some())?;
            selected_identity = Some(Extension::<u16>::decode(local_dw)?.into_data());
          }
          ExtensionTy::ServerName => {
            duplicated_error(server_name.is_some())?;
            server_name = Some(Extension::<ServerNameList>::decode(local_dw)?.into_data());
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
          ExtensionTy::ApplicationLayerProtocolNegotiation
          | ExtensionTy::ClientCertificateType
          | ExtensionTy::ServerCertificateType
          | ExtensionTy::EarlyData
          | ExtensionTy::Heartbeat
          | ExtensionTy::StatusRequest
          | ExtensionTy::UseSrtp => {
            return Err(TlsError::UnsupportedExtension.into());
          }
          _ => {
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
      cipher_suite,
      key_share: key_share_opt.ok_or_else(|| TlsError::MissingKeyShares)?,
      legacy_compression_method,
      legacy_session_id_echo,
      legacy_version,
      random,
      selected_identity,
      supported_versions,
    })
  }
}

impl Encode<De> for EncryptedExtensions<'_> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    self.legacy_version.encode(ew)?;
    ew.buffer().extend_from_slice(&self.random)?;
    ew.buffer().extend_from_slice(&self.legacy_session_id_echo)?;
    self.cipher_suite.encode(ew)?;
    ew.buffer().extend_from_byte(self.legacy_compression_method)?;
    Extension::new(ExtensionTy::PreSharedKey, self.selected_identity).encode(ew)?;
    Extension::new(ExtensionTy::KeyShare, &self.key_share).encode(ew)?;
    Extension::new(ExtensionTy::SupportedVersions, &self.supported_versions).encode(ew)?;
    Ok(())
  }
}
