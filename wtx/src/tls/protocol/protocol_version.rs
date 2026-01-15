use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::de::De,
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
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    Ok(Self::try_from(<u16 as Decode<De>>::decode(dw)?)?)
  }
}

impl Encode<De> for ProtocolVersion {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
