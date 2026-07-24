use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::{SingleTypeStorage, TryExtend},
  misc::Lease,
};

/// Optional field helper that delegates operations to the inner element, if any.
#[derive(Debug, Default, PartialEq)]
pub struct OptSeq<T, const TAG: u8>(
  /// Optional sequence
  pub T,
);

impl<'de, B, E, const TAG: u8> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>
  for OptSeq<Option<B>, TAG>
where
  B: Default + SingleTypeStorage<Item = E> + TryExtend<[E; 1]>,
  E: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    if dw.bytes.first().copied() == Some(TAG) {
      Ok(Self(Some(SequenceBuffer::<B>::decode(dw, TAG)?.0.0)))
    } else {
      Ok(Self(None))
    }
  }
}

impl<B, E, T, const TAG: u8> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for OptSeq<T, TAG>
where
  T: Lease<Option<B>> + SingleTypeStorage<Item = B>,
  B: Lease<[E]> + SingleTypeStorage<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapperAux>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    if let Some(elem) = self.0.lease() {
      SequenceBuffer(elem).encode(ew, ew.encode_aux.len_guess, TAG)?;
    }
    Ok(())
  }
}
