use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Len, SEQUENCE_TAG, SequenceBuffer},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::Vector,
  misc::Lease,
  x509::AccessDescription,
};

/// Indicates how to access information and services for the subject of the certificate in which
/// the extension appears.
#[derive(Debug, PartialEq)]
pub struct SubjectInformationAccess<B>(
  /// Entries
  pub Vector<AccessDescription<B>>,
);

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for SubjectInformationAccess<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(SequenceBuffer::decode(dw, SEQUENCE_TAG)?.0.0))
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for SubjectInformationAccess<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    SequenceBuffer(&self.0).encode(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG)
  }
}
