use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  misc::Lease,
};

/// Opaque ASN.1 object or element.
#[derive(Debug, PartialEq)]
pub struct Any<D> {
  tag: u8,
  len: Len,
  data: D,
}

impl<D> Any<D> {
  /// Generic data
  #[inline]
  pub const fn data(&self) -> &D {
    &self.data
  }

  /// Length of its associated data.
  pub const fn len(&self) -> &Len {
    &self.len
  }

  /// Identifier.
  pub const fn tag(&self) -> u8 {
    self.tag
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Any<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, len, data, rest) = decode_asn1_tlv(dw.bytes)?;
    dw.bytes = rest;
    Ok(Self { data, len, tag })
  }
}

impl<D> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Any<D>
where
  D: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ =
      ew.buffer.extend_from_copyable_slices([&[self.tag][..], &*self.len, self.data.lease()])?;
    Ok(())
  }
}
