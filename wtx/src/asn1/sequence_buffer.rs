use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SequenceDecodeCb, SequenceEncodeIter},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::TryExtend,
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
  E: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
{
  /// The encoding of an collection object requires the injection of a tag.
  #[inline]
  pub fn decode(
    dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>,
    tag: u8,
  ) -> crate::Result<(Self, &'de [u8])> {
    let mut extensions = B::default();
    let bytes = SequenceDecodeCb::new(|elem| {
      extensions.try_extend([elem])?;
      Ok(())
    })
    .decode(dw, tag)?;
    Ok((Self(extensions), bytes))
  }
}

impl<B, E> SequenceBuffer<B>
where
  B: Lease<[E]> + SingleTypeStorage<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapperAux>>,
{
  /// The encoding of an collection object requires the injection of a tag and the guessing of
  /// its entire length for performance reasons.
  #[inline]
  pub fn encode(
    &self,
    ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>,
    len_guess: Len,
    tag: u8,
  ) -> crate::Result<()> {
    SequenceEncodeIter(self.0.lease().iter()).encode(ew, len_guess, tag)
  }
}
