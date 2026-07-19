// https://datatracker.ietf.org/doc/html/rfc9846#section-4.2

use crate::{
  codec::{Decode, Encode},
  tls::{
    AlertDescription, TlsError, de::De, misc::tls_error_fatal,
    tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExtensionTy {
  ServerName = 0,
  MaxFragmentLength = 1,
  StatusRequest = 5,
  SupportedGroups = 10,
  SignatureAlgorithms = 13,
  UseSrtp = 14,
  Heartbeat = 15,
  ApplicationLayerProtocolNegotiation = 16,
  SignedCertificateTimestamp = 18,
  ClientCertificateType = 19,
  ServerCertificateType = 20,
  Padding = 21,
  PreSharedKey = 41,
  EarlyData = 42,
  SupportedVersions = 43,
  Cookie = 44,
  PskKeyExchangeModes = 45,
  CertificateAuthorities = 47,
  OidFilters = 48,
  PostHandshakeAuth = 49,
  SignatureAlgorithmsCert = 50,
  KeyShare = 51,
}

impl From<ExtensionTy> for u16 {
  #[inline]
  fn from(value: ExtensionTy) -> Self {
    match value {
      ExtensionTy::ServerName => 0,
      ExtensionTy::MaxFragmentLength => 1,
      ExtensionTy::StatusRequest => 5,
      ExtensionTy::SupportedGroups => 10,
      ExtensionTy::SignatureAlgorithms => 13,
      ExtensionTy::UseSrtp => 14,
      ExtensionTy::Heartbeat => 15,
      ExtensionTy::ApplicationLayerProtocolNegotiation => 16,
      ExtensionTy::SignedCertificateTimestamp => 18,
      ExtensionTy::ClientCertificateType => 19,
      ExtensionTy::ServerCertificateType => 20,
      ExtensionTy::Padding => 21,
      ExtensionTy::PreSharedKey => 41,
      ExtensionTy::EarlyData => 42,
      ExtensionTy::SupportedVersions => 43,
      ExtensionTy::Cookie => 44,
      ExtensionTy::PskKeyExchangeModes => 45,
      ExtensionTy::CertificateAuthorities => 47,
      ExtensionTy::OidFilters => 48,
      ExtensionTy::PostHandshakeAuth => 49,
      ExtensionTy::SignatureAlgorithmsCert => 50,
      ExtensionTy::KeyShare => 51,
    }
  }
}

impl TryFrom<u16> for ExtensionTy {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u16) -> crate::Result<Self> {
    Ok(match value {
      0 => Self::ServerName,
      1 => Self::MaxFragmentLength,
      5 => Self::StatusRequest,
      10 => Self::SupportedGroups,
      13 => Self::SignatureAlgorithms,
      14 => Self::UseSrtp,
      15 => Self::Heartbeat,
      16 => Self::ApplicationLayerProtocolNegotiation,
      18 => Self::SignedCertificateTimestamp,
      19 => Self::ClientCertificateType,
      20 => Self::ServerCertificateType,
      21 => Self::Padding,
      41 => Self::PreSharedKey,
      42 => Self::EarlyData,
      43 => Self::SupportedVersions,
      44 => Self::Cookie,
      45 => Self::PskKeyExchangeModes,
      47 => Self::CertificateAuthorities,
      48 => Self::OidFilters,
      49 => Self::PostHandshakeAuth,
      50 => Self::SignatureAlgorithmsCert,
      51 => Self::KeyShare,
      _ => return tls_error_fatal(TlsError::InvalidExtensionTy, AlertDescription::DecodeError),
    })
  }
}

impl<'de> Decode<'de, De> for ExtensionTy {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let tag: u16 = Decode::<'_, De>::decode(dw)?;
    Self::try_from(tag)
  }
}

impl Encode<De> for ExtensionTy {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    <u16 as Encode<De>>::encode(&u16::from(*self), ew)
  }
}
