use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  x509::SerialNumber,
};

/// A monotonically increasing sequence number for a given CRL scope and CRL issuer.
#[derive(Clone, Debug, PartialEq)]
pub struct CrlNumber(
  /// Identifier
  pub SerialNumber,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CrlNumber {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(SerialNumber::decode(dw)?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for CrlNumber {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
