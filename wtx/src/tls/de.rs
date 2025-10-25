use crate::{
  de::{DEController, Decode, Encode},
  misc::SuffixWriterMut,
  tls::TlsError,
};

pub(crate) struct De;

impl DEController for De {
  type Aux = ();
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

impl<'de> Decode<'de, De> for u16 {
  #[inline]
  #[track_caller]
  fn decode(_: &mut (), dw: &mut &'de [u8]) -> crate::Result<Self> {
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
  fn encode(&self, _: &mut (), sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    sw.extend_from_slice(&self.to_be_bytes())?;
    Ok(())
  }
}
