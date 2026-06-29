use crate::{
  asn1::{Asn1Error, MaxSizeTy},
  collections::ArrayVectorCopy,
  misc::int_conv::u8u16,
};
use core::{hint::unreachable_unchecked, ops::Deref};

/// Length in DER encoding
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Len(ArrayVectorCopy<u8, 3>);

impl Len {
  /// Instance with the value `1`.
  pub const ONE: Self = Self(ArrayVectorCopy::from_array([1]));
  /// Maximum length of the 1-byte long form.
  pub const MAX_ONE_BYTE: Self = Self(ArrayVectorCopy::from_array([127]));
  /// Maximum length of the 2-byte long form.
  pub const MAX_TWO_BYTES: Self = Self(ArrayVectorCopy::from_array([129, 255]));
  /// Maximum length of the 3-byte long form.
  pub const MAX_THREE_BYTES: Self = Self(ArrayVectorCopy::from_array([130, 255, 255]));
  /// Instance with the value `0`.
  pub const ZERO: Self = Self(ArrayVectorCopy::from_array([0]));

  /// Creates a length encoding from a `u8`, using short form (<=127) or long form (>127)
  #[inline]
  pub const fn from_u8(len: u8) -> Self {
    if len <= 127 {
      Self(ArrayVectorCopy::from_array([len]))
    } else {
      Self(ArrayVectorCopy::from_array([129, len]))
    }
  }

  /// Creates a 1, 2 or 3-byte long form length encoding from an `u16`.
  #[inline]
  pub fn from_u16(len: u16) -> Self {
    if let Ok(elem) = len.try_into() {
      Self::from_u8(elem)
    } else {
      let [b0, b1] = len.to_be_bytes();
      Self(ArrayVectorCopy::from_array([130, b0, b1]))
    }
  }

  /// Adds `additional` to `len` and encodes the result, returning an error if `len` > `u32::MAX`.
  #[inline]
  pub fn from_usize(additional: usize, mut len: usize) -> crate::Result<Len> {
    len = len.checked_add(additional).ok_or(Asn1Error::InvalidLen)?;
    if let Ok(elem) = u8::try_from(len) {
      Ok(Len::from_u8(elem))
    } else if let Ok(elem) = u16::try_from(len) {
      Ok(Len::from_u16(elem))
    } else {
      Err(Asn1Error::InvalidLen.into())
    }
  }

  /// Encoded representation
  #[inline]
  pub fn bytes(&self) -> &ArrayVectorCopy<u8, 3> {
    &self.0
  }

  /// Number representation.
  #[inline]
  pub fn size(&self) -> MaxSizeTy {
    match self.0.as_slice() {
      [b0] | [_, b0] => u8u16(*b0),
      [_, b0, b1] => MaxSizeTy::from_be_bytes([*b0, *b1]),
      // SAFETY: Constructors don't permit this branch
      _ => unsafe { unreachable_unchecked() },
    }
  }
}

impl Default for Len {
  #[inline]
  fn default() -> Self {
    Self::ZERO
  }
}

impl Deref for Len {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
