use crate::{
  codec::{CodecController, Decode, DecodeSeq, Encode},
  collection::Vector,
  misc::Lease,
};
use core::marker::PhantomData;

/// `D`ecode/`E`ncode
#[derive(Debug)]
pub struct GenericCodec<DRSR>(PhantomData<DRSR>);

impl<DRSR> CodecController for GenericCodec<DRSR> {
  type DecodeWrapper<'inner, 'outer, 'rem>
    = GenericDecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = GenericEncodeWrapper<'inner>
  where
    'inner: 'outer;
}

impl<DRSR> Decode<'_, GenericCodec<DRSR>> for () {
  #[inline]
  fn decode(_: &mut GenericDecodeWrapper<'_>) -> crate::Result<Self> {
    Ok(())
  }
}

impl<DRSR> DecodeSeq<'_, GenericCodec<DRSR>> for () {
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut GenericDecodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}

impl<DRSR> Encode<GenericCodec<DRSR>> for () {
  #[inline]
  fn encode(&self, _: &mut GenericEncodeWrapper<'_>) -> Result<(), crate::Error> {
    Ok(())
  }
}

/// Struct used for decoding different formats.
#[derive(Debug, PartialEq)]
pub struct GenericDecodeWrapper<'de> {
  pub(crate) bytes: &'de [u8],
}

impl<'de> GenericDecodeWrapper<'de> {
  /// New instance
  #[inline]
  pub const fn new(bytes: &'de [u8]) -> Self {
    Self { bytes }
  }
}

impl Lease<[u8]> for GenericDecodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

/// Struct used for encoding different formats.
#[derive(Debug)]
pub struct GenericEncodeWrapper<'any> {
  pub(crate) vector: &'any mut Vector<u8>,
}

impl<'any> GenericEncodeWrapper<'any> {
  /// New instance
  #[inline]
  pub const fn new(vector: &'any mut Vector<u8>) -> Self {
    Self { vector }
  }
}

impl Lease<[u8]> for GenericEncodeWrapper<'_> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.vector
  }
}
