use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SET_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::ArrayVectorU8,
  x509::AttributeTypeAndValue,
};

/// Unordered set of attribute type-value pairs.
#[derive(Debug, Default, PartialEq)]
pub struct RelativeDistinguishedName<'bytes> {
  /// Entries
  pub entries: ArrayVectorU8<AttributeTypeAndValue<'bytes>, 2>,
}

impl<'bytes> RelativeDistinguishedName<'bytes> {
  /// Shortcut
  pub const fn new(entries: ArrayVectorU8<AttributeTypeAndValue<'bytes>, 2>) -> Self {
    Self { entries }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for RelativeDistinguishedName<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self { entries: SequenceBuffer::decode(dw, SET_TAG)?.0 })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for RelativeDistinguishedName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.entries).encode(ew, Len::MAX_ONE_BYTE, SET_TAG)
  }
}
