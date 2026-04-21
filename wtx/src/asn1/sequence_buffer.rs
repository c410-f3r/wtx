use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SequenceDecodeCb, SequenceEncodeIter},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::TryExtend,
  misc::{Lease, SingleTypeStorage},
};

/// Helper that collects sequences of the same type
#[derive(Debug, PartialEq)]
pub struct SequenceBuffer<B>(
  /// Buffer
  pub B,
);

impl<'de, B, E> SequenceBuffer<B>
where
  B: Default + SingleTypeStorage<Item = E> + TryExtend<[E; 1]>,
  E: Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  /// The encoding of an collection object requires the injection of a tag.
  #[inline]
  pub fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>, tag: u8) -> crate::Result<Self> {
    let mut extensions = B::default();
    SequenceDecodeCb::new(|elem| {
      extensions.try_extend([elem])?;
      Ok(())
    })
    .decode(dw, tag)?;
    Ok(Self(extensions))
  }
}

impl<B, E> SequenceBuffer<B>
where
  B: Lease<[E]> + SingleTypeStorage<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapper>>,
{
  /// The encoding of an collection object requires the injection of a tag and the guessing of
  /// its entire length for performance reasons.
  pub fn encode(
    &self,
    ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>,
    len_guess: Len,
    tag: u8,
  ) -> crate::Result<()> {
    SequenceEncodeIter(self.0.lease().iter()).encode(ew, len_guess, tag)
  }
}
