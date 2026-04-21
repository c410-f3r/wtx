use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::Vector,
  x509::Attribute,
};

/// Used to convey identification attributes (e.g., nationality) of the subject.
#[derive(Debug, PartialEq)]
pub struct SubjectDirectoryAttributes<'bytes>(
  /// A sequence of attributes.
  pub Vector<Attribute<'bytes, 2>>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SubjectDirectoryAttributes<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for SubjectDirectoryAttributes<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)
  }
}
