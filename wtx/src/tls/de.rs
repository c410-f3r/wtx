use crate::{
  de::{DEController, Decode, Encode},
  tls::{TlsError, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

pub(crate) struct De;

impl DEController for De {
  type DecodeWrapper<'inner, 'outer, 'misc>
    = DecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'misc>
    = EncodeWrapper<'inner>
  where
    'inner: 'outer;
}

impl<'de> Decode<'de, De> for &'de [u8] {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(dw.bytes())
  }
}

impl<'de> Decode<'de, De> for u8 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [a, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU8Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(*a)
  }
}

impl<'de> Decode<'de, De> for u16 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [a, b, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidU16Prefix.into());
    };
    *dw.bytes_mut() = rest;
    Ok(u16::from_be_bytes([*a, *b]))
  }
}

impl<'de, const N: usize> Decode<'de, De> for [u8; N] {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let Some((lhs, rhs)) = dw.bytes().split_at_checked(N) else {
      return Err(TlsError::InvalidArray.into());
    };
    *dw.bytes_mut() = rhs;
    Ok(lhs.try_into()?)
  }
}

impl Encode<De> for u16 {
  #[inline]
  #[track_caller]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slice(&self.to_be_bytes())?;
    Ok(())
  }
}
