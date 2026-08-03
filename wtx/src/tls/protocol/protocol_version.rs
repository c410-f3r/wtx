use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// TLS version
pub enum ProtocolVersion {
  /// TLS 1.0
  Tls1 = 0x0301,
  /// TLS 1.1
  Tls11 = 0x0302,
  /// TLS 1.2
  Tls12 = 0x0303,
  /// TLS 1.3
  Tls13 = 0x0304,
}

impl From<ProtocolVersion> for u16 {
  #[inline]
  fn from(value: ProtocolVersion) -> Self {
    match value {
      ProtocolVersion::Tls1 => 0x0301,
      ProtocolVersion::Tls11 => 0x0302,
      ProtocolVersion::Tls12 => 0x0303,
      ProtocolVersion::Tls13 => 0x0304,
    }
  }
}

impl TryFrom<u16> for ProtocolVersion {
  type Error = crate::Error;
  #[inline]
  fn try_from(value: u16) -> crate::Result<Self> {
    Ok(match value {
      0x0301 => ProtocolVersion::Tls1,
      0x0302 => ProtocolVersion::Tls11,
      0x0303 => ProtocolVersion::Tls12,
      0x0304 => ProtocolVersion::Tls13,
      _ => return Err(TlsError::UnknownProtocolVersion.into()),
    })
  }
}

impl<'de> Decode<'de, De> for ProtocolVersion {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Self::try_from(<u16 as Decode<De>>::decode(dw)?)
  }
}

impl Encode<De> for ProtocolVersion {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
