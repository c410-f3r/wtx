use crate::{
  de::{Decode, Encode},
  tls::{TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameType {
  HostName = 0,
}

impl<'de> Decode<'de, De> for NameType {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::UnknownNameType.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self::HostName)
  }
}

impl Encode<De> for NameType {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_byte(0)?;
    Ok(())
  }
}
