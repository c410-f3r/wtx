use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::GeneralNames,
};

/// The issuer alternative name extension allows identities to be bound to the issuer
/// of the certificate.
#[derive(Debug, PartialEq)]
pub struct IssuerAlternativeName<'bytes> {
  /// See [`GeneralNames`].
  pub general_names: GeneralNames<'bytes>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for IssuerAlternativeName<'de> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self { general_names: GeneralNames::decode(dw)? })
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for IssuerAlternativeName<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.general_names.encode(ew)
  }
}
