use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, INTEGER_TAG, Len, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorU8,
  misc::Lease as _,
  x509::X509Error,
};
use core::ops::Deref;

const MAX_LEN: usize = 20;

/// Serial Number in DER encoding. Can contain up to 20 bytes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SerialNumber(ArrayVectorU8<u8, MAX_LEN>);

impl SerialNumber {
  /// Internal bytes
  #[inline]
  pub const fn bytes(&self) -> &ArrayVectorU8<u8, MAX_LEN> {
    &self.0
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for SerialNumber {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (INTEGER_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(X509Error::InvalidSerialNumberBytes.into());
    };
    let serial_number = SerialNumber::try_from(value)?;
    dw.bytes = rest;
    Ok(serial_number)
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for SerialNumber {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
    if value == [0] || value.len() > MAX_LEN {
      return Err(X509Error::InvalidSerialNumberBytes.into());
    }
    Ok(Self(ArrayVectorU8::try_from(value)?))
  }
}
