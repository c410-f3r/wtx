use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Integer},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
};

/// A monotonically increasing sequence number for a given CRL scope and CRL issuer.
#[derive(Debug, PartialEq)]
pub struct CrlNumber<'bytes>(
  /// See [`Integer`].
  pub Integer<&'bytes [u8]>,
);

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for CrlNumber<'de> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    Ok(Self(Integer::decode(dw)?))
  }
}

impl<'bytes> Encode<GenericCodec<(), Asn1EncodeWrapper>> for CrlNumber<'bytes> {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    self.0.encode(ew)
  }
}
