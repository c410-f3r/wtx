use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameType {
  HostName = 0,
}

impl<'de> Decode<'de, De> for NameType {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::UnknownNameType.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self::HostName)
  }
}

impl Encode<De> for NameType {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(0)
  }
}
