use crate::{
  data_transformation::dnsn::{DecodeWrapper, EncodeWrapper},
  misc::{Decode, DecodeSeq, Encode},
};

/// `D`ecode/`E`ncode
#[derive(Debug)]
pub struct De<DRSR>(core::marker::PhantomData<DRSR>);

impl<DRSR> crate::misc::DEController for De<DRSR> {
  type Aux = DRSR;
  type DecodeWrapper<'inner, 'outer>
    = DecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer>
    = EncodeWrapper<'inner>
  where
    'inner: 'outer;
}

impl<DRSR> Decode<'_, De<DRSR>> for () {
  #[inline]
  fn decode(_: &mut DRSR, _: &mut DecodeWrapper<'_>) -> crate::Result<Self> {
    Ok(())
  }
}

impl<DRSR> DecodeSeq<'_, De<DRSR>> for () {
  #[inline]
  fn decode_seq(
    _: &mut DRSR,
    _: &mut crate::misc::Vector<Self>,
    _: &mut DecodeWrapper<'_>,
  ) -> crate::Result<()> {
    Ok(())
  }
}

impl<DRSR> Encode<De<DRSR>> for () {
  #[inline]
  fn encode(&self, _: &mut DRSR, _: &mut EncodeWrapper<'_>) -> Result<(), crate::Error> {
    Ok(())
  }
}
