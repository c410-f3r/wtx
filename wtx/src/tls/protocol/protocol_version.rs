use crate::{
  de::{Decode, Encode},
  tls::{de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

create_enum! {
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  /// TLS version
  pub enum ProtocolVersion<u16> {
    /// TLS 1.2
    Tls12 = (0x0303),
    /// TLS 1.3
    Tls13 = (0x0304)
  }
}

impl<'de> Decode<'de, De> for ProtocolVersion {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self::try_from(<u16 as Decode<De>>::decode(dw)?)?)
  }
}

impl Encode<De> for ProtocolVersion {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
