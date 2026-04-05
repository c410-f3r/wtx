use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, BIT_STRING_TAG, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  misc::Lease,
};

/// Arbitrary-length sequence of binary digits. Not to be confused with UTF-8 strings.
#[derive(Debug, PartialEq)]
pub struct BitString<B> {
  bytes: B,
  unused_bits: u8,
}

impl<B> BitString<B>
where
  B: Lease<[u8]>,
{
  /// New instance without unused bits.
  #[inline]
  pub const fn from_bytes(bytes: B) -> Self {
    Self { unused_bits: 0, bytes }
  }

  /// New instance from all parameters.
  #[inline]
  pub fn new(bytes: B, unused_bits: u8) -> crate::Result<Self> {
    check_unused_bits(unused_bits, bytes.lease())?;
    Ok(Self { bytes, unused_bits })
  }

  /// Unsafe version of [`Self::new`]
  ///
  /// # SAFETY
  ///
  /// The unused bits of the last byte of `bytes` must match `unused_bits`
  #[inline]
  pub const unsafe fn new_unchecked(bytes: B, unused_bits: u8) -> Self {
    Self { bytes, unused_bits }
  }

  /// The raw bytes carrying the bit data.
  #[inline]
  pub const fn bytes(&self) -> &B {
    &self.bytes
  }

  /// Returns the number of **padding bits** in the final octet of the underlying payload.
  #[inline]
  pub const fn unused_bits(&self) -> u8 {
    self.unused_bits
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for BitString<&'de [u8]> {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    let actual_tag = dw.decode_aux.tag.unwrap_or(BIT_STRING_TAG);
    let (true, [unused_bits, bytes @ ..]) = (tag == actual_tag, value) else {
      return Err(Asn1Error::InvalidBitString.into());
    };
    check_unused_bits(*unused_bits, bytes)?;
    dw.bytes = rest;
    Ok(Self { bytes, unused_bits: *unused_bits })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapper>> for BitString<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[ew.encode_aux.tag.unwrap_or(BIT_STRING_TAG)][..],
      &*Len::from_usize(1, self.bytes.lease().len())?,
      &[self.unused_bits],
      self.bytes.lease(),
    ])?;
    Ok(())
  }
}

#[inline]
fn check_unused_bits(unused_bits: u8, bytes: &[u8]) -> crate::Result<()> {
  if unused_bits > 7 || (bytes.is_empty() && unused_bits != 0) {
    return Err(Asn1Error::InvalidBitString.into());
  }
  if unused_bits > 0
    && let [.., last] = bytes
    && *last & (1u8 << unused_bits).wrapping_sub(1) != 0
  {
    return Err(Asn1Error::InvalidBitString.into());
  }
  Ok(())
}
