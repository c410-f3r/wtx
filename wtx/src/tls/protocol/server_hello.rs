// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.3

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  rng::CryptoRng,
  tls::{
    TlsError,
    cipher_suite::CipherSuiteTy,
    de::De,
    misc::u16_chunk,
    protocol::{
      key_share_entry::KeyShareEntry, protocol_version::ProtocolVersion,
      protocol_versions::SupportedVersions, server_hello_extension::ServerHelloExtension,
      server_hello_extension_ty::ServerHelloExtensionTy,
    },
  },
};

#[derive(Debug)]
pub(crate) struct ServerHello<'any> {
  cipher_suite: CipherSuiteTy,
  key_share: KeyShareEntry<'any>,
  legacy_compression_method: u8,
  legacy_session_id_echo: ArrayVectorU8<u8, 32>,
  legacy_version: ProtocolVersion,
  random: [u8; 32],
  selected_identity: Option<u16>,
  supported_versions: SupportedVersions,
}

impl<'any> ServerHello<'any> {
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

  pub(crate) fn key_share(&self) -> &KeyShareEntry<'any> {
    &self.key_share
  }
}

impl<'de> Decode<'de, De> for ServerHello<'de> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32]>::decode(dw)?;
    let legacy_session_id_echo =
      u16_chunk(dw, TlsError::InvalidLegacySessionIdEcho, |el| Ok(*el))?.try_into()?;
    let cipher_suite = CipherSuiteTy::decode(dw)?;
    let legacy_compression_method = <u8 as Decode<'de, De>>::decode(dw)?;

    let mut key_share_opt = None;
    let mut selected_identity = None;
    let mut supported_versions_opt = None;
    u16_chunk(dw, TlsError::InvalidServerHelloLen, |bytes| {
      while !bytes.is_empty() {
        let extension_ty = {
          let tmp_bytes = &mut *bytes;
          ServerHelloExtensionTy::decode(tmp_bytes)?
        };
        match extension_ty {
          ServerHelloExtensionTy::PreSharedKey => {
            selected_identity = Some(ServerHelloExtension::<u16>::decode(bytes)?.into_data());
          }
          ServerHelloExtensionTy::SupportedVersions => {
            supported_versions_opt =
              Some(ServerHelloExtension::<SupportedVersions>::decode(bytes)?.into_data());
          }
          ServerHelloExtensionTy::KeyShare => {
            key_share_opt = Some(KeyShareEntry::decode(bytes)?);
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

impl Encode<De> for ServerHello<'_> {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    self.legacy_version.encode(ew)?;
    ew.extend_from_slice(&self.random)?;
    ew.extend_from_slice(&self.legacy_session_id_echo)?;
    self.cipher_suite.encode(ew)?;
    ew.extend_from_byte(self.legacy_compression_method)?;
    if let Some(elem) = self.selected_identity {
      ServerHelloExtension::new(ServerHelloExtensionTy::PreSharedKey, elem).encode(ew)?;
    }
    ServerHelloExtension::new(ServerHelloExtensionTy::KeyShare, &self.key_share).encode(ew)?;
    ServerHelloExtension::new(ServerHelloExtensionTy::SupportedVersions, &self.supported_versions)
      .encode(ew)?;
    Ok(())
  }
}
