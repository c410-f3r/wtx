use crate::{
  misc::{DEController, Decode, Encode, SuffixWriterMut},
  tls::TlsError,
};

pub(crate) struct De;

impl DEController for De {
  type DecodeWrapper<'any, 'de> = &'de [u8];
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer>
    = SuffixWriterMut<'inner>
  where
    'inner: 'outer;
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

impl Encode<De> for u16 {
  #[inline]
  #[track_caller]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slice(&self.to_be_bytes())?;
    Ok(())
  }
}
