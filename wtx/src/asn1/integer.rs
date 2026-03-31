use crate::{
  asn1::{Asn1Error, INTEGER_TAG, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  misc::Lease,
};

/// Big-endian two's complement bytes
#[derive(Debug, PartialEq)]
pub struct Integer<B>(B);

impl<B> Integer<B>
where
  B: Lease<[u8]>,
{
  /// New instance that checks invalid data.
  pub fn new(bytes: B) -> crate::Result<Self> {
    check_bytes(bytes.lease())?;
    Ok(Self(bytes))
  }

  /// Bytes
  pub fn bytes(&self) -> &[u8] {
    self.0.lease()
  }
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Integer<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (INTEGER_TAG, _, bytes, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidInteger.into());
    };
    check_bytes(bytes)?;
    dw.bytes = rest;
    Ok(Self(bytes))
  }
}

impl<B> Encode<GenericCodec<(), ()>> for Integer<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    let bytes = self.0.lease();
    let _ = ew.buffer.extend_from_copyable_slices([
      &[INTEGER_TAG][..],
      &*Len::from_usize(0, self.0.lease().len())?,
      bytes,
    ])?;
    Ok(())
  }
}

fn check_bytes(bytes: &[u8]) -> crate::Result<()> {
  match bytes {
    [] => Err(Asn1Error::InvalidInteger.into()),
    [0, b, ..] if b & 0b1000_0000 == 0 => Err(Asn1Error::InvalidInteger.into()),
    [0b1111_1111, b, ..] if b & 0b1000_0000 != 0 => Err(Asn1Error::InvalidInteger.into()),
    _ => Ok(()),
  }
}
