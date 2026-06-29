use crate::{
  codec::Decode,
  misc::Usize,
  tls::{TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper},
};

pub(crate) struct U24(u32);

impl<'de> Decode<'de, De> for U24 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, b1, b2, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU16Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self(u32::from_be_bytes([*b0, *b1, *b2, 0])))
  }
}

impl From<U24> for usize {
  #[inline]
  fn from(value: U24) -> Self {
    Usize::from(value.0).into_usize()
  }
}
