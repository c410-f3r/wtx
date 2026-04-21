use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SET_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::TryExtend,
  misc::{Lease, SingleTypeStorage},
};

/// A collection of elements.
#[derive(Debug, PartialEq)]
pub struct Set<S>(
  /// A collection of elements.
  pub S,
);

impl<'de, S> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Set<S>
where
  S: Default + SingleTypeStorage + TryExtend<[S::Item; 1]>,
  S::Item: Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SET_TAG)?.0))
  }
}

impl<S> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Set<S>
where
  S: Lease<[S::Item]> + SingleTypeStorage,
  S::Item: Encode<GenericCodec<(), Asn1EncodeWrapper>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SET_TAG)
  }
}
