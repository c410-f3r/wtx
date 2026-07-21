use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SET_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::{SingleTypeStorage, TryExtend},
  misc::Lease,
};

/// A collection of elements.
#[derive(Debug, PartialEq)]
pub struct Set<S>(
  /// A collection of elements.
  pub S,
);

impl<'de, S> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Set<S>
where
  S: Default + SingleTypeStorage + TryExtend<[S::Item; 1]>,
  S::Item: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SET_TAG)?.0.0))
  }
}

impl<S> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Set<S>
where
  S: Lease<[S::Item]> + SingleTypeStorage,
  S::Item: Encode<GenericCodec<(), Asn1EncodeWrapperAux>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SET_TAG)
  }
}
