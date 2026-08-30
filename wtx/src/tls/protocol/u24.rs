use crate::{
  codec::Decode,
  misc::Usize,
  tls::{TlsError, tls_cc::TlsCc, tls_cc_wrappers::TlsDecodeWrapper},
};

#[derive(Debug)]
pub(crate) struct U24(u32);

impl<'de> Decode<'de, TlsCc> for U24 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, b1, b2, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU24Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self(u32::from_be_bytes([0, *b0, *b1, *b2])))
  }
}

impl From<U24> for usize {
  #[inline]
  fn from(value: U24) -> Self {
    Usize::from(value.0).into_usize()
  }
}
