use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError,
    tls_cc::TlsCc,
    tls_cc_wrappers::{TlsDecodeWrapper, TlsEncodeWrapper},
  },
};

#[derive(Debug)]
pub(crate) struct PskKeyExchangeModes {}

impl<'de> Decode<'de, TlsCc> for PskKeyExchangeModes {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [1, 1, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidPskKeyExchangeMode.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self {})
  }
}

impl Encode<TlsCc> for PskKeyExchangeModes {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&[1, 1])?;
    Ok(())
  }
}
