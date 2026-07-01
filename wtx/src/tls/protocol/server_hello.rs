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
      protocol_version::ProtocolVersion, protocol_versions::SupportedVersions,
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
  supported_versions: SupportedVersions,
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
      supported_versions: SupportedVersions::new(ArrayVectorCopy::from_array([
        ProtocolVersion::Tls13,
      ])),
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
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32] as Decode<'de, De>>::decode(dw)?;
    let is_hello_retry_request = random == HELLO_RETRY_REQUEST;
    let legacy_session_id_echo =
      u8_chunk(dw, TlsError::InvalidLegacySessionIdEcho, |el| Ok(el.bytes()))?.try_into()?;
    let cipher_suite = CipherSuite::decode(dw)?;
    let legacy_compression_method = <u8 as Decode<'de, De>>::decode(dw)?;
    let mut key_share_opt = None;
    let mut selected_identity_opt = None;
    let mut supported_versions_opt = None;
    u16_chunk(dw, TlsError::InvalidServerHelloLen, |local_dw| {
      while !local_dw.bytes().is_empty() {
        let extension_ty = {
          let begin_bytes = local_dw.bytes();
          let extension_ty = ExtensionTy::decode(local_dw)?;
          *local_dw.bytes_mut() = begin_bytes;
          extension_ty
        };
        match extension_ty {
          ExtensionTy::Cookie => {
            if is_hello_retry_request {
              return Err(TlsError::UnsupportedExtension.into());
            }
            return Err(TlsError::MismatchedExtension.into());
          }
          ExtensionTy::KeyShare => {
            *local_dw.is_hello_retry_request_mut() = is_hello_retry_request;
            let rslt = KeyShareEntry::decode(local_dw);
            *local_dw.is_hello_retry_request_mut() = false;
            key_share_opt = Some(rslt?);
          }
          ExtensionTy::PreSharedKey => {
            if is_hello_retry_request {
              return Err(TlsError::MismatchedExtension.into());
            }
            selected_identity_opt = Some(Extension::<u16>::decode(local_dw)?.into_data());
          }
          ExtensionTy::SupportedVersions => {
            supported_versions_opt =
              Some(Extension::<SupportedVersions>::decode(local_dw)?.into_data());
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
      is_hello_retry_request,
      key_share: key_share_opt.ok_or(TlsError::MissingKeyShares)?,
      legacy_compression_method,
      legacy_session_id_echo,
      legacy_version,
      random,
      selected_identity: selected_identity_opt,
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
      if !self.is_hello_retry_request {
        Extension::new(ExtensionTy::PreSharedKey, self.selected_identity).encode(local_ew)?;
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
