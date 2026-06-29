use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, INTEGER_TAG, Len, decode_asn1_tlv,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::ArrayVectorCopy,
  misc::Lease as _,
};
use core::{hint::unreachable_unchecked, ops::Deref};

/// `u32` in DER encoding
///
/// Not meant to be used for public keys or serial numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct U32(ArrayVectorCopy<u8, 5>);

impl U32 {
  /// Instance with the value `1`
  pub const ONE: Self = Self::from_u8(1);

  /// Creates an integer encoding from a `u8`, using short form (<=127) or long form (>127)
  #[inline]
  pub const fn from_u8(value: u8) -> Self {
    if value <= 127 {
      Self(ArrayVectorCopy::from_array([value]))
    } else {
      Self(ArrayVectorCopy::from_array([0, value]))
    }
  }

  /// Creates a 1-byte, 2-bytes or 3-bytes long form integer encoding from an `u16`.
  #[inline]
  pub fn from_u16(value: u16) -> Self {
    if let Ok(elem) = value.try_into() {
      Self::from_u8(elem)
    } else {
      let [b0, b1] = value.to_be_bytes();
      if value <= 32767 {
        Self(ArrayVectorCopy::from_array([b0, b1]))
      } else {
        Self(ArrayVectorCopy::from_array([0, b0, b1]))
      }
    }
  }

  /// Creates a 1-byte, 2-bytes, 3-bytes or 4-bytes long form integer encoding from an `u32`.
  #[inline]
  pub fn from_u32(value: u32) -> Self {
    if let Ok(elem) = u8::try_from(value) {
      Self::from_u8(elem)
    } else if let Ok(elem) = u16::try_from(value) {
      Self::from_u16(elem)
    } else {
      let [b0, b1, b2, b3] = value.to_be_bytes();
      if value <= 16_777_215 {
        if value <= 8_388_607 {
          Self(ArrayVectorCopy::from_array([b1, b2, b3]))
        } else {
          Self(ArrayVectorCopy::from_array([0, b1, b2, b3]))
        }
      } else {
        if value <= 2_147_483_647 {
          Self(ArrayVectorCopy::from_array([b0, b1, b2, b3]))
        } else {
          Self(ArrayVectorCopy::from_array([0, b0, b1, b2, b3]))
        }
      }
    }
  }

  /// Returns an error if `value` > `u32::MAX`.
  #[inline]
  pub fn from_usize(value: usize) -> crate::Result<Self> {
    Ok(Self::from_u32(u32::try_from(value)?))
  }

  /// Interprets the internal bytes as `u32`.
  #[inline]
  pub fn u32(&self) -> u32 {
    match self.0.as_slice() {
      [b0] | [0, b0] => u32::from(*b0),
      [b0, b1] | [0, b0, b1] => u32::from(u16::from_be_bytes([*b0, *b1])),
      [b0, b1, b2] | [0, b0, b1, b2] => u32::from_be_bytes([0, *b0, *b1, *b2]),
      [b0, b1, b2, b3] | [0, b0, b1, b2, b3] => u32::from_be_bytes([*b0, *b1, *b2, *b3]),
      // SAFETY: Constructors ensure valid lengths
      _ => unsafe { unreachable_unchecked() },
    }
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for U32 {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let actual_tag = dw.decode_aux.tag.unwrap_or(INTEGER_TAG);
    let (tag, _, value, rest) = decode_asn1_tlv(dw.bytes)?;
    if tag != actual_tag {
      return Err(Asn1Error::InvalidInteger.into());
    }
    let rslt = U32::try_from(value)?;
    dw.bytes = rest;
    Ok(rslt)
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for U32 {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    let actual_tag = ew.encode_aux.tag.unwrap_or(INTEGER_TAG);
    let _ = ew.buffer.extend_from_copyable_slices([
      &[actual_tag][..],
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
      [b0 @ 0..=127] | [0, b0 @ 128..=255] => Self::from_u32(u32::from(*b0)),
      [b0 @ 1..=127, b1] | [0, b0 @ 128..=255, b1] => {
        Self::from_u32(u16::from_be_bytes([*b0, *b1]).into())
      }
      [b0 @ 1..=127, b1, b2] | [0, b0 @ 128..=255, b1, b2] => {
        Self::from_u32(u32::from_be_bytes([0, *b0, *b1, *b2]))
      }
      [b0 @ 1..=127, b1, b2, b3] | [0, b0 @ 128..=255, b1, b2, b3] => {
        Self::from_u32(u32::from_be_bytes([*b0, *b1, *b2, *b3]))
      }
      _ => return Err(Asn1Error::InvalidU32Bytes.into()),
    })
  }
}
