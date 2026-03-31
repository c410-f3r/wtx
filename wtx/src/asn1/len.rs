use crate::{asn1::Asn1Error, collection::ArrayVectorU8};
use core::{hint::unreachable_unchecked, ops::Deref};

/// Length of a object
#[derive(Debug, PartialEq)]
pub struct Len(ArrayVectorU8<u8, 3>);

impl Len {
  /// Maximum length of the 1-byte long form.
  pub const MAX_ONE: Self = Self(ArrayVectorU8::from_array_u8([127]));
  /// Maximum length of the 2-byte long form.
  pub const MAX_TWO: Self = Self(ArrayVectorU8::from_array_u8([129, 255]));
  /// Maximum length of the 3-byte long form.
  pub const MAX_THREE: Self = Self(ArrayVectorU8::from_array_u8([130, 255, 255]));

  /// Creates a length encoding from a `u8`, using short form (<=127) or long form (>127)
  #[inline]
  pub const fn from_u8(len: u8) -> Self {
    if len <= 127 {
      Self(ArrayVectorU8::from_array_u8([len]))
    } else {
      Self(ArrayVectorU8::from_array_u8([129, len]))
    }
  }

  /// Creates a 3-byte long form length encoding from an `u16`.
  #[inline]
  pub const fn from_u16(len: u16) -> Self {
    let [a, b] = len.to_be_bytes();
    Self(ArrayVectorU8::from_array_u8([130, a, b]))
  }

  /// Adds `additional` to `len` and encodes the result, returning an error if > `u16::MAX`.
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

  /// Length in `u8`.
  #[inline]
  pub fn len_u8(&self) -> u8 {
    self.0.len()
  }

  /// Number representation.
  #[inline]
  pub fn num(&self) -> u16 {
    match self.0.as_slice() {
      [a] => u8::from_be_bytes([*a]).into(),
      [_, b] => u8::from_be_bytes([*b]).into(),
      [_, b, c] => u16::from_be_bytes([*b, *c]),
      // SAFETY: Constructors don't permit this branch
      _ => unsafe { unreachable_unchecked() },
    }
  }
}

impl Deref for Len {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
