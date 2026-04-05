use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  x509::extensions::CrlDistributionPoints,
};

/// Identifies how delta CRL information is obtained.
#[derive(Debug, PartialEq)]
pub struct FreshestCrl<'bytes>(pub CrlDistributionPoints<'bytes>);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for FreshestCrl<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(CrlDistributionPoints::decode(dw)?))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for FreshestCrl<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
