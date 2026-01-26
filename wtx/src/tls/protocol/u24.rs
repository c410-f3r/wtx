use crate::{
  de::Decode,
  misc::Usize,
  tls::{TlsError, de::De, decode_wrapper::DecodeWrapper},
};

pub(crate) struct U24(u32);

impl<'de> Decode<'de, De> for U24 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [a, b, c, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU16Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self(u32::from_be_bytes([*a, *b, *c, 0])))
  }
}

impl From<U24> for usize {
  fn from(value: U24) -> Self {
    Usize::from(value.0).into_usize()
  }
}
