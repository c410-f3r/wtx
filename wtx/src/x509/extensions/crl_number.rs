use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::SerialNumber,
};

/// A monotonically increasing sequence number for a given CRL scope and CRL issuer.
#[derive(Clone, Debug, PartialEq)]
pub struct CrlNumber(
  /// Identifier
  pub SerialNumber,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for CrlNumber {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(SerialNumber::decode(dw)?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for CrlNumber {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
