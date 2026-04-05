use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, Len, asn1_writer, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
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
  pub fn decode(
    dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>,
    tag: u8,
  ) -> crate::Result<Self> {
    let (local_tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if local_tag != tag {
      return Err(Asn1Error::InvalidGenericSequence(local_tag, tag).into());
    }
    dw.bytes = value;
    let mut extensions = B::default();
    while !dw.bytes.is_empty() {
      extensions.try_extend([E::decode(dw)?])?;
    }
    dw.bytes = rest;
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
    ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>,
    len_guess: Len,
    tag: u8,
  ) -> crate::Result<()> {
    ew.encode_aux.len_guess = len_guess;
    let rslt = asn1_writer(ew, ew.encode_aux.len_guess.clone(), tag, |local_ew| {
      for elem in self.0.lease() {
        elem.encode(local_ew)?;
      }
      Ok(())
    });
    ew.encode_aux.len_guess = Len::default();
    rslt
  }
}
