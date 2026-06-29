use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, GeneralizedTime},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
};

/// Provides the date on which it is known or suspected that the private key was compromised or
/// that the certificate otherwise became invalid.
#[derive(Debug, PartialEq)]
pub struct InvalidityDate(
  /// The date of invalidity.
  pub GeneralizedTime,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for InvalidityDate {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(GeneralizedTime::decode(dw)?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for InvalidityDate {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
