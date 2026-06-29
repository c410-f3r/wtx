use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
  x509::extensions::CrlDistributionPoints,
};

/// Identifies how delta CRL information is obtained.
#[derive(Debug, PartialEq)]
pub struct FreshestCrl<B>(pub CrlDistributionPoints<B>);

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for FreshestCrl<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    Ok(Self(CrlDistributionPoints::decode(dw)?))
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for FreshestCrl<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
