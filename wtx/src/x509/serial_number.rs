use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, INTEGER_TAG, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::ArrayVectorU8,
  misc::Lease,
};
use core::ops::Deref;

/// Serial Number in DER encoding. Can contain up to 20 bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct SerialNumber(ArrayVectorU8<u8, 20>);

impl SerialNumber {
  /// Internal bytes
  #[inline]
  pub const fn bytes(&self) -> &ArrayVectorU8<u8, 20> {
    &self.0
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for SerialNumber {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (INTEGER_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidInteger.into());
    };
    let value = SerialNumber::try_from(value)?;
    dw.bytes = rest;
    Ok(value)
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for SerialNumber {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[INTEGER_TAG][..],
      &*Len::from_u8(self.0.len()),
      self.0.lease(),
    ])?;
    Ok(())
  }
}

impl Deref for SerialNumber {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl TryFrom<&[u8]> for SerialNumber {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let ([1..=127, ..] | [0, 128..=255, ..]) = value else {
      return Err(Asn1Error::InvalidSerialNumberBytes.into());
    };
    Ok(Self(ArrayVectorU8::try_from(value)?))
  }
}
