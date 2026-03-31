use core::marker::PhantomData;

use crate::{
  codec::{CodecController, Decode, DecodeSeq, Encode},
  collection::Vector,
  misc::Lease,
};

/// Generic `D`ecoder/`E`ncoder
#[derive(Debug)]
pub struct GenericCodec<DA, EA> {
  decode_aux: PhantomData<DA>,
  encode_aux: PhantomData<EA>,
}

impl<DA, EA> CodecController for GenericCodec<DA, EA> {
  type DecodeWrapper<'inner, 'outer, 'rem>
    = GenericDecodeWrapper<'inner, DA>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = GenericEncodeWrapper<'inner, EA>
  where
    'inner: 'outer;
}

impl<DA, EA> Decode<'_, GenericCodec<DA, EA>> for () {
  #[inline]
  fn decode(_: &mut GenericDecodeWrapper<'_, DA>) -> crate::Result<Self> {
    Ok(())
  }
}

impl<DA, EA> DecodeSeq<'_, GenericCodec<DA, EA>> for () {
  #[inline]
  fn decode_seq(_: &mut Vector<Self>, _: &mut GenericDecodeWrapper<'_, DA>) -> crate::Result<()> {
    Ok(())
  }
}

impl<DA, EA> Encode<GenericCodec<DA, EA>> for () {
  #[inline]
  fn encode(&self, _: &mut GenericEncodeWrapper<'_, EA>) -> Result<(), crate::Error> {
    Ok(())
  }
}

/// Struct used for decoding different formats.
#[derive(Debug, PartialEq)]
pub struct GenericDecodeWrapper<'de, DA> {
  /// Raw bytes where decoded elements are originated.
  pub bytes: &'de [u8],
  /// Auxiliary decoding data
  pub decode_aux: DA,
}

impl<'de, DA> GenericDecodeWrapper<'de, DA> {
  /// Shortcut
  #[inline]
  pub const fn new(bytes: &'de [u8], decode_aux: DA) -> Self {
    Self { bytes, decode_aux }
  }
}

impl<DA> Lease<[u8]> for GenericDecodeWrapper<'_, DA> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.bytes
  }
}

/// Struct used for encoding different formats.
#[derive(Debug)]
pub struct GenericEncodeWrapper<'any, EA> {
  /// Buffer where the encoded contents are stored.
  pub buffer: &'any mut Vector<u8>,
  /// Auxiliary encoding data
  pub encode_aux: EA,
}

impl<'any, EA> GenericEncodeWrapper<'any, EA> {
  /// Shortcut
  #[inline]
  pub const fn new(buffer: &'any mut Vector<u8>, encode_aux: EA) -> Self {
    Self { buffer, encode_aux }
  }
}

impl<EA> Lease<[u8]> for GenericEncodeWrapper<'_, EA> {
  #[inline]
  fn lease(&self) -> &[u8] {
    self.buffer
  }
}
