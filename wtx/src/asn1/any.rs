use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, Len, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Opaque ASN.1 object or element.
#[derive(Clone, Debug, PartialEq)]
pub struct Any<D> {
  tag: u8,
  len: Len,
  data: D,
}

impl<'any> Any<&'any [u8]> {
  /// New instance from arbitrary bytes
  //
  // FIXME(stable): constant results
  #[inline]
  pub fn from_bytes(tag: u8, data: &'any [u8]) -> crate::Result<Self> {
    Ok(Self::from_bytes_opt(tag, data).ok_or(Asn1Error::InvalidAnyBytes)?)
  }

  /// Constant version of [`Self::from_bytes`].
  #[inline]
  pub const fn from_bytes_opt(tag: u8, data: &'any [u8]) -> Option<Self> {
    let data_len = data.len();
    if data_len > 255 {
      return None;
    }
    Some(Self { tag, len: Len::from_u8(data_len as u8), data })
  }
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
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
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
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ =
      ew.buffer.extend_from_copyable_slices([&[self.tag][..], &*self.len, self.data.lease()])?;
    Ok(())
  }
}
