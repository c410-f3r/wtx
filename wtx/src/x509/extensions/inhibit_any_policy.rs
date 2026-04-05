use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, U32},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// Indicates that the special anyPolicy OID, with the value { 2 5 29 32 0 }, is not
/// considered an explicit match for other certificate policies except when it appears
/// in an intermediate self-issued CA certificate.
#[derive(Debug, PartialEq)]
pub struct InhibitAnyPolicy(U32);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for InhibitAnyPolicy {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(U32::decode(dw)?))
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for InhibitAnyPolicy {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)?;
    Ok(())
  }
}
