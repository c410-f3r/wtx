use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, BIT_STRING_TAG, Len, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  misc::Lease,
};

/// Arbitrary-length sequence of binary digits. Not to be confused with UTF-8 strings.
#[derive(Clone, Debug, PartialEq)]
pub struct BitString<B> {
  bytes: B,
  unused_bits: u8,
}

impl<B> BitString<B>
where
  B: Lease<[u8]>,
{
  /// New instance from arbitrary bytes
  #[inline]
  pub fn from_bytes(bytes: B) -> Self {
    let unused_bits = if let [.., last] = bytes.lease() { last.trailing_zeros() % 8 } else { 0 };
    Self { bytes, unused_bits: unused_bits.try_into().unwrap_or_default() }
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

  /// Returns the inner elements
  #[inline]
  pub fn into_parts(self) -> (B, u8) {
    (self.bytes, self.unused_bits)
  }

  /// Returns the number of **padding bits** in the final octet of the underlying payload.
  #[inline]
  pub const fn unused_bits(&self) -> u8 {
    self.unused_bits
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for BitString<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let actual_tag = dw.decode_aux.tag.unwrap_or(BIT_STRING_TAG);
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    let (true, [unused_bits, bytes @ ..]) = (tag == actual_tag, value) else {
      return Err(Asn1Error::InvalidBitString.into());
    };
    check_unused_bits(*unused_bits, bytes)?;
    dw.bytes = rest;
    Ok(Self { bytes: bytes.try_into().map_err(Into::into)?, unused_bits: *unused_bits })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for BitString<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let actual_tag = ew.encode_aux.tag.unwrap_or(BIT_STRING_TAG);
    let _ = ew.buffer.extend_from_copyable_slices([
      &[actual_tag][..],
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
