use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, INTEGER_TAG, Len, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Big-endian two's complement bytes
#[derive(Clone, Debug, PartialEq)]
pub struct Integer<B>(B);

impl<B> Integer<B>
where
  B: Lease<[u8]>,
{
  /// New instance that checks invalid data.
  #[inline]
  pub fn new(bytes: B) -> crate::Result<Self> {
    check_bytes(bytes.lease())?;
    Ok(Self(bytes))
  }

  /// Bytes
  #[inline]
  pub fn bytes(&self) -> &[u8] {
    self.0.lease()
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Integer<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (INTEGER_TAG, _, bytes, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidInteger.into());
    };
    check_bytes(bytes)?;
    dw.bytes = rest;
    Ok(Self(bytes.try_into().map_err(Into::into)?))
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Integer<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
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
    [0, b0, ..] if b0 & 0b1000_0000 == 0 => Err(Asn1Error::InvalidInteger.into()),
    [0b1111_1111, b0, ..] if b0 & 0b1000_0000 != 0 => Err(Asn1Error::InvalidInteger.into()),
    _ => Ok(()),
  }
}
