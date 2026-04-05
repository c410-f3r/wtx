use crate::{
  asn1::{Asn1DecodeWrapper, Asn1EncodeWrapper, Asn1Error, INTEGER_TAG, Len, decode_asn1_tlv},
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::ArrayVectorU8,
  misc::Lease,
};
use core::{hint::unreachable_unchecked, ops::Deref};

/// `u32` in DER encoding
///
/// Not meant to be used for public keys or serial numbers.
#[derive(Debug, PartialEq)]
pub struct U32(ArrayVectorU8<u8, 5>);

impl U32 {
  /// Instance with the value `1`
  pub const ONE: Self = Self::from_u8(1);

  /// Creates an integer encoding from a `u8`, using short form (<=127) or long form (>127)
  #[inline]
  pub const fn from_u8(value: u8) -> Self {
    if value <= 127 {
      Self(ArrayVectorU8::from_array_u8([value]))
    } else {
      Self(ArrayVectorU8::from_array_u8([0, value]))
    }
  }

  /// Creates a 1-byte, 2-bytes or 3-bytes long form integer encoding from an `u16`.
  #[inline]
  pub fn from_u16(value: u16) -> Self {
    if let Ok(elem) = value.try_into() {
      Self::from_u8(elem)
    } else {
      let [a, b] = value.to_be_bytes();
      if value <= 32767 {
        Self(ArrayVectorU8::from_array_u8([a, b]))
      } else {
        Self(ArrayVectorU8::from_array_u8([0, a, b]))
      }
    }
  }

  /// Creates a 1-byte, 2-bytes, 3-bytes or 4-bytes long form integer encoding from an `u32`.
  #[inline]
  pub fn from_u32(value: u32) -> Self {
    if let Ok(elem) = value.try_into() {
      Self::from_u8(elem)
    } else if let Ok(elem) = value.try_into() {
      Self::from_u16(elem)
    } else {
      let [a, b, c, d] = value.to_be_bytes();
      if value <= 16_777_215 {
        if value <= 8_388_607 {
          Self(ArrayVectorU8::from_array_u8([b, c, d]))
        } else {
          Self(ArrayVectorU8::from_array_u8([0, b, c, d]))
        }
      } else {
        if value <= 2_147_483_647 {
          Self(ArrayVectorU8::from_array_u8([a, b, c, d]))
        } else {
          Self(ArrayVectorU8::from_array_u8([0, a, b, c, d]))
        }
      }
    }
  }

  /// Returns an error if `value` > `u32::MAX`.
  #[inline]
  pub fn from_usize(value: usize) -> crate::Result<Self> {
    Ok(Self::from_u32(u32::try_from(value)?))
  }

  /// Interprets the internal bytes as [`MaxIntTy`].
  #[inline]
  pub fn u32(&self) -> u32 {
    match self.0.as_slice() {
      [a] => u32::from(*a),
      [0, b] => u32::from(*b),
      [a, b] => u32::from(u16::from_be_bytes([*a, *b])),
      [0, a, b] => u32::from(u16::from_be_bytes([*a, *b])),
      [a, b, c] => u32::from_be_bytes([0, *a, *b, *c]),
      [0, a, b, c] => u32::from_be_bytes([0, *a, *b, *c]),
      [a, b, c, d] => u32::from_be_bytes([*a, *b, *c, *d]),
      [0, a, b, c, d] => u32::from_be_bytes([*a, *b, *c, *d]),
      // SAFETY: Constructors ensure valid lengths
      _ => unsafe { unreachable_unchecked() },
    }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for U32 {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let (INTEGER_TAG, _, value, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidInteger.into());
    };
    let value = U32::try_from(value)?;
    dw.bytes = rest;
    Ok(value)
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for U32 {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let _ = ew.buffer.extend_from_copyable_slices([
      &[INTEGER_TAG][..],
      &*Len::from_u8(self.0.len()),
      self.0.lease(),
    ])?;
    Ok(())
  }
}

impl Deref for U32 {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl TryFrom<&[u8]> for U32 {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    Ok(match value {
      [a @ 0..=127] => Self::from_u32(u32::from(*a)),
      [0, a @ 128..=255] => Self::from_u32(u32::from(*a)),
      [a @ 1..=127, b] => Self::from_u32(u16::from_be_bytes([*a, *b]).into()),
      [0, a @ 128..=255, b] => Self::from_u32(u16::from_be_bytes([*a, *b]).into()),
      [a @ 1..=127, b, c] => Self::from_u32(u32::from_be_bytes([0, *a, *b, *c])),
      [0, a @ 128..=255, b, c] => Self::from_u32(u32::from_be_bytes([0, *a, *b, *c])),
      [a @ 1..=127, b, c, d] => Self::from_u32(u32::from_be_bytes([*a, *b, *c, *d])),
      [0, a @ 128..=255, b, c, d] => Self::from_u32(u32::from_be_bytes([*a, *b, *c, *d])),
      _ => return Err(Asn1Error::InvalidU32Bytes.into()),
    })
  }
}
