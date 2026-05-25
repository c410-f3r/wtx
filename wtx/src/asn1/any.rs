use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, Len, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Opaque ASN.1 object or element.
#[derive(Clone, Debug, PartialEq)]
pub struct Any<D> {
  bytes: D,
  tag: u8,
  len: Len,
}

impl<'bytes> Any<&'bytes [u8]> {
  /// Doesn't perform checks that ensure that `len` is equal to the length of `bytes`.
  //
  // FIXME(STABLE): Return `Result`
  #[inline]
  pub const fn new(bytes: &'bytes [u8], tag: u8, len: Len) -> Self {
    Self { bytes, tag, len }
  }
}

impl<D> Any<D> {
  /// The whole slice that contains the tag, the length and the data.
  #[inline]
  pub const fn bytes(&self) -> &D {
    &self.bytes
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

impl<D> Any<D>
where
  D: Lease<[u8]>,
{
  /// Generic data
  #[inline]
  pub fn data(&self) -> &[u8] {
    let skip = 1u8.wrapping_add(self.len.bytes().len());
    // SAFETY: All instances are constructors with checks that ensure at least `skip` bytes
    unsafe { self.bytes.lease().get(skip.into()..).unwrap_unchecked() }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for Any<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, len, data, _) = decode_asn1_tlv(dw.bytes)?;
    let idx = 1usize.wrapping_add(len.bytes().len().into()).wrapping_add(data.len());
    let Some((lhs, rhs)) = dw.bytes.split_at_checked(idx) else {
      return Err(Asn1Error::InvalidAnyBytes.into());
    };
    dw.bytes = rhs;
    Ok(Self { bytes: lhs, len, tag })
  }
}

impl<D> Encode<GenericCodec<(), Asn1EncodeWrapper>> for Any<D>
where
  D: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ =
      ew.buffer.extend_from_copyable_slices([&[self.tag][..], &*self.len, self.bytes.lease()])?;
    Ok(())
  }
}
