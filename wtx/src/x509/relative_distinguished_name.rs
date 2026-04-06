use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SET_TAG, SequenceBuffer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::ArrayVectorU32,
  x509::AttributeTypeAndValue,
};

/// Unordered set of attribute type-value pairs.
#[derive(Debug, Default, PartialEq)]
pub struct RelativeDistinguishedName<'bytes>(
  /// Entries
  pub ArrayVectorU32<AttributeTypeAndValue<'bytes>, 2>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for RelativeDistinguishedName<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SET_TAG)?.0))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for RelativeDistinguishedName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SET_TAG)
  }
}
