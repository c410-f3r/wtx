use crate::de::{
  Decode, DecodeSeq, Encode,
  format::{DecodeWrapper, EncodeWrapper},
};

/// `D`ecode/`E`ncode
#[derive(Debug)]
pub struct De<DRSR>(core::marker::PhantomData<DRSR>);

impl<DRSR> crate::de::DEController for De<DRSR> {
  type DecodeWrapper<'inner, 'outer, 'rem>
    = DecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = EncodeWrapper<'inner>
  where
    'inner: 'outer;
}

impl<DRSR> Decode<'_, De<DRSR>> for () {
  #[inline]
  fn decode(_: &mut DecodeWrapper<'_>) -> crate::Result<Self> {
    Ok(())
  }
}

impl<DRSR> DecodeSeq<'_, De<DRSR>> for () {
  #[inline]
  fn decode_seq(
    _: &mut crate::collection::Vector<Self>,
    _: &mut DecodeWrapper<'_>,
  ) -> crate::Result<()> {
    Ok(())
  }
}

impl<DRSR> Encode<De<DRSR>> for () {
  #[inline]
  fn encode(&self, _: &mut EncodeWrapper<'_>) -> Result<(), crate::Error> {
    Ok(())
  }
}
