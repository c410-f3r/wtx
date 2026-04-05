use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SequenceBuffer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::TryExtend,
  misc::{Lease, SingleTypeStorage},
};

/// Optional field helper that delegates operations to the inner element, if any.
#[derive(Debug, Default, PartialEq)]
pub struct Opt<T>(
  /// Optional element
  pub T,
);

impl<'de, T> Opt<Option<T>>
where
  T: Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  /// The decoding of an optional object requires the injection of a tag.
  pub fn decode(
    dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>,
    tag: u8,
  ) -> crate::Result<Self> {
    if dw.bytes.first().copied() == Some(tag) {
      dw.decode_aux.tag = Some(tag);
      let rslt = T::decode(dw);
      dw.decode_aux.tag = None;
      Ok(Self(Some(rslt?)))
    } else {
      Ok(Self(None))
    }
  }
}

impl<'de, B, E> Opt<Option<B>>
where
  B: Default + SingleTypeStorage<Item = E> + TryExtend<[E; 1]>,
  E: Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  /// The decoding of an optional collection requires the injection of a tag.
  pub fn decode_seq(
    dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>,
    tag: u8,
  ) -> crate::Result<Self> {
    if dw.bytes.first().copied() == Some(tag) {
      Ok(Self(Some(SequenceBuffer::<B>::decode(dw, tag)?.0)))
    } else {
      Ok(Self(None))
    }
  }
}

impl<E, T> Opt<T>
where
  T: Lease<Option<E>> + SingleTypeStorage<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapper>>,
{
  /// The encoding of an optional object requires the injection of a tag.
  pub fn encode(
    &self,
    ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>,
    tag: u8,
  ) -> crate::Result<()> {
    if let Some(elem) = self.0.lease() {
      ew.encode_aux.tag = Some(tag);
      let rslt = elem.encode(ew);
      ew.encode_aux.tag = None;
      rslt?;
    }
    Ok(())
  }
}

impl<B, E, T> Opt<T>
where
  T: Lease<Option<B>> + SingleTypeStorage<Item = B>,
  B: Lease<[E]> + SingleTypeStorage<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapper>>,
{
  /// The encoding of an optional object requires the injection of a tag and the guessing of
  /// its entire length for performance reasons.
  pub fn encode_seq(
    &self,
    ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>,
    len_guess: Len,
    tag: u8,
  ) -> crate::Result<()> {
    if let Some(elem) = self.0.lease() {
      SequenceBuffer(elem).encode(ew, len_guess, tag)?;
    }
    Ok(())
  }
}
