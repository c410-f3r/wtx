use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError,
    tls_cc::TlsCc,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
  },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameType {
  HostName = 0,
}

impl<'de> Decode<'de, TlsCc> for NameType {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [0, rest @ ..] = dw.bytes() else {
      return Err(TlsError::UnknownNameType.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self::HostName)
  }
}

impl Encode<TlsCc> for NameType {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().push(0)
  }
}
