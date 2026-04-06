use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::AccessDescription,
};

/// Indicates how to access information and services for the issuer of the certificate in which
/// the extension appears.
#[derive(Debug, PartialEq)]
pub struct AuthorityInformationAccess<'bytes>(
  /// Entries
  pub Vector<AccessDescription<'bytes>>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for AuthorityInformationAccess<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for AuthorityInformationAccess<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)
  }
}
