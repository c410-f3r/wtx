use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::{ArrayVectorU8, TryExtend, Vector},
  misc::{Lease, SingleTypeStorage},
  x509::RelativeDistinguishedName,
};

/// [`Name`] composed by an [`ArrayVectorU8`].
pub type NameArrayVector<'de, const N: usize> =
  Name<ArrayVectorU8<RelativeDistinguishedName<'de>, N>>;
/// [`Name`] composed by a [`Vector`].
pub type NameVector<'de> = Name<Vector<RelativeDistinguishedName<'de>>>;

/// Distinguished name.
#[derive(Debug, PartialEq)]
pub struct Name<R> {
  /// Entries
  pub rdn_sequence: R,
}

impl<R> Name<R> {
  /// Shortcut
  pub const fn new(rdn_sequence: R) -> Self {
    Self { rdn_sequence }
  }
}

impl<'de, E, R> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Name<R>
where
  R: Default + SingleTypeStorage<Item = E> + TryExtend<[E; 1]>,
  E: Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self { rdn_sequence: SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0 })
  }
}

impl<E, R> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Name<R>
where
  R: Lease<[E]> + SingleTypeStorage<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapper>>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.rdn_sequence).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}
