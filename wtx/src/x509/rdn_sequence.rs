use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::RelativeDistinguishedName,
};

/// Unordered set of attribute type-value pairs.
#[derive(Debug, Default, PartialEq)]
pub struct RdnSequence<'bytes>(
  /// Entries
  pub Vector<RelativeDistinguishedName<'bytes>>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for RdnSequence<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for RdnSequence<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}
