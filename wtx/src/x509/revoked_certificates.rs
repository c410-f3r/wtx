use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::Vector,
  misc::Lease,
  x509::RevokedCertificate,
};

/// List of revoked certificates
#[derive(Clone, Debug, PartialEq)]
pub struct RevokedCertificates<B>(
  /// List of revoked certificates
  pub Vector<RevokedCertificate<B>>,
);

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for RevokedCertificates<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let collection = SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0.0;
    Ok(Self(collection))
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for RevokedCertificates<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG)
  }
}
