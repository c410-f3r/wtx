use crate::{
  de::{DEController, Decode, Encode},
  misc::SuffixWriterMut,
  tls::TlsError,
};

pub(crate) struct De;

impl DEController for De {
  type DecodeWrapper<'inner, 'outer, 'misc>
    = &'inner [u8]
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'misc>
    = SuffixWriterMut<'inner>
  where
    'inner: 'outer;
}

impl<'de> Decode<'de, De> for u8 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let [a, rest @ ..] = dw else {
      return Err(TlsError::InvalidU8Prefix.into());
    };
    *dw = rest;
    Ok(*a)
  }
}

impl<'de> Decode<'de, De> for u16 {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let [a, b, rest @ ..] = dw else {
      return Err(TlsError::InvalidU16Prefix.into());
    };
    *dw = rest;
    Ok(u16::from_be_bytes([*a, *b]))
  }
}

impl<'de, const N: usize> Decode<'de, De> for [u8; N] {
  #[inline]
  #[track_caller]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let Some((lhs, rhs)) = dw.split_at_checked(N) else {
      return Err(TlsError::InvalidArray.into());
    };
    *dw = rhs;
    Ok(lhs.try_into()?)
  }
}

impl Encode<De> for u16 {
  #[inline]
  #[track_caller]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    sw.extend_from_slice(&self.to_be_bytes())?;
    Ok(())
  }
}
