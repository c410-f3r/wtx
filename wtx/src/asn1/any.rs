use crate::{
  asn1::{Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  misc::Lease,
};

/// Opaque BER/DER header and its associated content.
#[derive(Debug, PartialEq)]
pub struct Any<B> {
  data: B,
  len: Len,
  tag: u8,
}

impl<'de> Decode<'de, GenericCodec<Option<u8>, ()>> for Any<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Option<u8>>) -> crate::Result<Self> {
    let (tag, len, data, rest) = decode_asn1_tlv(dw.bytes)?;
    dw.bytes = rest;
    Ok(Self { data, len, tag })
  }
}

impl<B> Encode<GenericCodec<(), ()>> for Any<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, ()>) -> crate::Result<()> {
    let _ =
      ew.buffer.extend_from_copyable_slices([&[self.tag][..], &*self.len, self.data.lease()])?;
    Ok(())
  }
}
