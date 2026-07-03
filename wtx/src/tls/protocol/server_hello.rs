// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.3

use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  misc::counter_writer::{CounterWriterBytesTy, u16_write},
  rng::CryptoRng,
  tls::{
    CipherSuite, HELLO_RETRY_REQUEST, TlsError,
    de::De,
    misc::{u8_chunk, u16_chunk},
    protocol::{
      extension::Extension, extension_ty::ExtensionTy, key_share_entry::KeyShareEntry,
      protocol_version::ProtocolVersion, protocol_versions::SupportedVersionsServer,
    },
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Debug)]
pub(crate) struct ServerHello<'any> {
  cipher_suite: CipherSuite,
  is_hello_retry_request: bool,
  key_share: KeyShareEntry<&'any [u8]>,
  legacy_compression_method: u8,
  legacy_session_id_echo: ArrayVectorCopy<u8, 32>,
  legacy_version: ProtocolVersion,
  random: [u8; 32],
  selected_identity: Option<u16>,
  supported_versions: SupportedVersionsServer,
}

impl<'any> ServerHello<'any> {
  pub(crate) fn new<RNG>(
    cipher_suite: CipherSuite,
    is_hello_retry_request: bool,
    key_share: KeyShareEntry<&'any [u8]>,
    legacy_session_id_echo: ArrayVectorCopy<u8, 32>,
    rng: &mut RNG,
    selected_identity: Option<u16>,
  ) -> Self
  where
    RNG: CryptoRng,
  {
    let random = if is_hello_retry_request {
      HELLO_RETRY_REQUEST
    } else {
      let mut random = [0u8; 32];
      rng.fill_slice(&mut random);
      random
    };
    Self {
      cipher_suite,
      is_hello_retry_request,
      key_share,
      legacy_compression_method: 0,
      legacy_session_id_echo,
      legacy_version: ProtocolVersion::Tls12,
      random,
      selected_identity,
      supported_versions: SupportedVersionsServer::new(ProtocolVersion::Tls13),
    }
  }

  pub(crate) fn cipher_suite(&self) -> CipherSuite {
    self.cipher_suite
  }

  pub(crate) fn key_share(&self) -> &KeyShareEntry<&'any [u8]> {
    &self.key_share
  }
}

impl<'de> Decode<'de, De> for ServerHello<'de> {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let err = TlsError::InvalidServerHello;
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32] as Decode<'de, De>>::decode(dw)?;
    let is_hello_retry_request = random == HELLO_RETRY_REQUEST;
    let legacy_session_id_echo = u8_chunk(dw, err, |el| Ok(el.bytes()))?.try_into()?;
    let cipher_suite = CipherSuite::decode(dw)?;
    let legacy_compression_method = <u8 as Decode<'de, De>>::decode(dw)?;
    let mut key_share_opt = None;
    let mut selected_identity = None;
    let mut supported_versions_opt = None;
    u16_chunk(dw, err, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = ExtensionTy::decode(local_dw)?;
        u16_chunk(local_dw, err, |local_local_dw| {
          manage_extension(
            local_local_dw,
            extension_ty,
            is_hello_retry_request,
            &mut key_share_opt,
            &mut selected_identity,
            &mut supported_versions_opt,
          )
        })?;
      }
      Ok(())
    })?;
    let Some(supported_versions) = supported_versions_opt else {
      return Err(TlsError::MissingSupportedVersions.into());
    };
    if supported_versions.selected_version != ProtocolVersion::Tls13 {
      return Err(TlsError::UnsupportedTlsVersion.into());
    }
    Ok(Self {
      cipher_suite,
      is_hello_retry_request,
      key_share: key_share_opt.ok_or(TlsError::MissingKeyShares)?,
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
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    self.legacy_version.encode(ew)?;
    let _ = ew.buffer().extend_from_copyable_slices([
      &self.random,
      &[self.legacy_session_id_echo.len()][..],
      &self.legacy_session_id_echo,
    ])?;
    self.cipher_suite.encode(ew)?;
    ew.buffer().push(self.legacy_compression_method)?;
    u16_write(CounterWriterBytesTy::IgnoresLen, None, ew, |local_ew| {
      if !self.is_hello_retry_request
        && let Some(identity) = self.selected_identity
      {
        Extension::new(ExtensionTy::PreSharedKey, identity).encode(local_ew)?;
      }
      {
        *local_ew.is_hello_retry_request_mut() = self.is_hello_retry_request;
        let rslt = Extension::new(ExtensionTy::KeyShare, &self.key_share).encode(local_ew);
        *local_ew.is_hello_retry_request_mut() = false;
        rslt?;
      }
      Extension::new(ExtensionTy::SupportedVersions, &self.supported_versions).encode(local_ew)?;
      Ok(())
    })
  }
}

#[inline]
fn manage_extension<'de>(
  dw: &mut TlsDecodeWrapper<'de>,
  extension_ty: ExtensionTy,
  is_hello_retry_request: bool,
  key_share_opt: &mut Option<KeyShareEntry<&'de [u8]>>,
  selected_identity_opt: &mut Option<u16>,
  supported_versions_opt: &mut Option<SupportedVersionsServer>,
) -> crate::Result<()> {
  match extension_ty {
    ExtensionTy::Cookie => {
      if is_hello_retry_request {
        return Err(TlsError::UnsupportedExtension.into());
      }
      return Err(TlsError::MismatchedExtension.into());
    }
    ExtensionTy::KeyShare => {
      *dw.is_hello_retry_request_mut() = is_hello_retry_request;
      let rslt = KeyShareEntry::decode(dw);
      *dw.is_hello_retry_request_mut() = false;
      *key_share_opt = Some(rslt?);
    }
    ExtensionTy::PreSharedKey => {
      if is_hello_retry_request {
        return Err(TlsError::MismatchedExtension.into());
      }
      *selected_identity_opt = Some(<u16 as Decode<'_, De>>::decode(dw)?);
    }
    ExtensionTy::SupportedVersions => {
      *supported_versions_opt = Some(SupportedVersionsServer::decode(dw)?);
    }
    ExtensionTy::ApplicationLayerProtocolNegotiation
    | ExtensionTy::CertificateAuthorities
    | ExtensionTy::ClientCertificateType
    | ExtensionTy::EarlyData
    | ExtensionTy::Heartbeat
    | ExtensionTy::MaxFragmentLength
    | ExtensionTy::OidFilters
    | ExtensionTy::Padding
    | ExtensionTy::PostHandshakeAuth
    | ExtensionTy::PskKeyExchangeModes
    | ExtensionTy::ServerCertificateType
    | ExtensionTy::ServerName
    | ExtensionTy::SignatureAlgorithms
    | ExtensionTy::SignatureAlgorithmsCert
    | ExtensionTy::SignedCertificateTimestamp
    | ExtensionTy::StatusRequest
    | ExtensionTy::SupportedGroups
    | ExtensionTy::UseSrtp => {
      return Err(TlsError::MismatchedExtension.into());
    }
  }
  Ok(())
}
