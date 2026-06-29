use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::GeneralNames,
};

/// The issuer alternative name extension allows identities to be bound to the issuer
/// of the certificate.
#[derive(Debug, PartialEq)]
pub struct IssuerAlternativeName<B> {
  /// See [`GeneralNames`].
  pub general_names: GeneralNames<B>,
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for IssuerAlternativeName<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self { general_names: GeneralNames::decode(dw)? })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for IssuerAlternativeName<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.general_names.encode(ew)
  }
}
