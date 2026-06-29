use crate::{
  asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, Len, decode_asn1_tlv},
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Opaque ASN.1 object or element.
#[derive(Clone, Debug, PartialEq)]
pub struct Any<B> {
  bytes: B,
  tag: u8,
  len: Len,
}

impl<B> Any<B> {
  /// Doesn't perform checks that ensure that `len` is equal to the length of `bytes`.
  //
  // FIXME(STABLE): Return `Result`
  #[inline]
  pub const fn new(bytes: B, tag: u8, len: Len) -> Self {
    Self { bytes, tag, len }
  }

  /// The whole slice that contains the tag, the length and the data.
  #[inline]
  pub const fn bytes(&self) -> &B {
    &self.bytes
  }

  /// Length of its associated data.
  #[inline]
  pub const fn len(&self) -> &Len {
    &self.len
  }

  /// Identifier.
  #[inline]
  pub const fn tag(&self) -> u8 {
    self.tag
  }
}

impl<B> Any<B>
where
  B: Lease<[u8]>,
{
  /// Generic data
  #[inline]
  pub fn data(&self) -> &[u8] {
    let skip = 1u8.wrapping_add(self.len.bytes().len());
    // SAFETY: All instances are constructors with checks that ensure at least `skip` bytes
    unsafe { self.bytes.lease().get(skip.into()..).unwrap_unchecked() }
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Any<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (tag, len, data, _) = decode_asn1_tlv(dw.bytes)?;
    let idx = 1usize.wrapping_add(len.bytes().len().into()).wrapping_add(data.len());
    let Some((lhs, rhs)) = dw.bytes.split_at_checked(idx) else {
      return Err(Asn1Error::InvalidAnyBytes.into());
    };
    dw.bytes = rhs;
    Ok(Self { bytes: lhs.try_into().map_err(Into::into)?, len, tag })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Any<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let _ =
      ew.buffer.extend_from_copyable_slices([&[self.tag][..], &*self.len, self.bytes.lease()])?;
    Ok(())
  }
}
