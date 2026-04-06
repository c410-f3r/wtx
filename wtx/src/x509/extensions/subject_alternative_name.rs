use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::GeneralNames,
};

/// Allows identities to be bound to the subject of the certificate.
#[derive(Debug, PartialEq)]
pub struct SubjectAlternativeName<'bytes>(
  /// See [`GeneralNames`]
  pub GeneralNames<'bytes>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SubjectAlternativeName<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(GeneralNames::decode(dw)?))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for SubjectAlternativeName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
