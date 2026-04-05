use crate::{
  asn1::{Asn1Error, MaxSizeTy},
  collection::ArrayVectorU8,
};
use core::{hint::unreachable_unchecked, ops::Deref};

/// Length in DER encoding
#[derive(Clone, Debug, PartialEq)]
pub struct Len(ArrayVectorU8<u8, 3>);

impl Len {
  /// Instance with the value `1`.
  pub const ONE: Self = Self(ArrayVectorU8::from_array_u8([1]));
  /// Maximum length of the 1-byte long form.
  pub const MAX_ONE_BYTE: Self = Self(ArrayVectorU8::from_array_u8([127]));
  /// Maximum length of the 2-byte long form.
  pub const MAX_TWO_BYTES: Self = Self(ArrayVectorU8::from_array_u8([129, 255]));
  /// Maximum length of the 3-byte long form.
  pub const MAX_THREE_BYTES: Self = Self(ArrayVectorU8::from_array_u8([130, 255, 255]));
  /// Instance with the value `0`.
  pub const ZERO: Self = Self(ArrayVectorU8::from_array_u8([0]));

  /// Creates a length encoding from a `u8`, using short form (<=127) or long form (>127)
  #[inline]
  pub const fn from_u8(len: u8) -> Self {
    if len <= 127 {
      Self(ArrayVectorU8::from_array_u8([len]))
    } else {
      Self(ArrayVectorU8::from_array_u8([129, len]))
    }
  }

  /// Creates a 1, 2 or 3-byte long form length encoding from an `u16`.
  #[inline]
  pub fn from_u16(len: u16) -> Self {
    if let Ok(elem) = len.try_into() {
      Self::from_u8(elem)
    } else {
      let [a, b] = len.to_be_bytes();
      Self(ArrayVectorU8::from_array_u8([130, a, b]))
    }
  }

  /// Adds `additional` to `len` and encodes the result, returning an error if `len` > `u32::MAX`.
  #[inline]
  pub fn from_usize(additional: usize, mut len: usize) -> crate::Result<Len> {
    len = len.checked_add(additional).ok_or(Asn1Error::InvalidLen)?;
    if let Ok(len) = u8::try_from(len) {
      Ok(Len::from_u8(len))
    } else if let Ok(len) = u16::try_from(len) {
      Ok(Len::from_u16(len))
    } else {
      Err(Asn1Error::InvalidLen.into())
    }
  }

  /// Number representation.
  #[inline]
  pub fn size(&self) -> MaxSizeTy {
    match self.0.as_slice() {
      [a] => MaxSizeTy::from(*a),
      [_, b] => MaxSizeTy::from(*b),
      [_, b, c] => MaxSizeTy::from_be_bytes([*b, *c]),
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
