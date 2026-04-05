use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::RevokedCertificate,
};

/// List of revoked certificates
#[derive(Debug, PartialEq)]
pub struct RevokedCertificates<'bytes>(
  /// List of revoked certificates
  pub Vector<RevokedCertificate<'bytes>>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for RevokedCertificates<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let collection = SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0;
    Ok(Self(collection))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for RevokedCertificates<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG)
  }
}
