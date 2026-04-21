// `Usize` only permits platforms with pointer sizes of 32bits or 64bits.

macro_rules! u8_cap {
  () => {
    255
  };
}
macro_rules! u16_cap {
  () => {
    65_535
  };
}
#[cfg(target_pointer_width = "64")]
macro_rules! u32_cap {
  () => {
    4_294_967_295
  };
}
macro_rules! u64_cap {
  () => {
    9_223_372_036_854_775_807
  };
}
#[cfg(target_pointer_width = "64")]
macro_rules! usize_cap {
  () => {
    9_223_372_036_854_775_807
  };
}

#[cfg(not(target_pointer_width = "64"))]
macro_rules! u32_cap {
  () => {
    2_147_483_647
  };
}
#[cfg(not(target_pointer_width = "64"))]
macro_rules! usize_cap {
  () => {
    2_147_483_647
  };
}

use crate::misc::{LeaseMut, TryArithmetic, Usize};
use core::fmt::Display;

/// Determines how many elements can be stored in a linear collection.
pub trait LinearStorageLen:
  Copy
  + Display
  + Default
  + Eq
  + From<u8>
  + Ord
  + PartialEq
  + PartialOrd
  + Sized
  + TryArithmetic<Self, Output = Self>
{
  /// The size of this length in bits.
  const BITS: u8;
  /// The size of this length in bytes.
  const BYTES: u8 = Self::BITS / 8;
  /// If the maximum number of allowed elements is backed by an `u64` primitive.
  const IS_UPPER_BOUND_U64: bool = Self::UPPER_BOUND_USIZE == u64_cap!();
  /// The maximum number of allowed elements.
  const UPPER_BOUND: Self;
  /// The maximum number of allowed elements as `usize`.
  const UPPER_BOUND_USIZE: usize;
  /// Instance that represents the number one.
  const ONE: Self;
  /// Instance that represents the number zero.
  const ZERO: Self;

  /// Array of bytes that compose this length
  type Array: Default + LeaseMut<[u8]>;

  /// Returns the memory representation of this integer as a byte array in little-endian byte order.
  fn from_le_bytes(array: Self::Array) -> Self;

  /// Tries to create a new instance from a `usize` primitive.
  fn from_usize(num: usize) -> crate::Result<Self>;

  /// Returns the memory representation of this integer as a byte array in little-endian byte order.
  fn to_le_bytes(self) -> Self::Array;

  /// Converts itself into `usize`.
  fn usize(self) -> usize;

  /// Wrapping (modular) addition.
  #[must_use]
  fn wrapping_add(self, rhs: Self) -> Self;

  /// Wrapping (modular) subtraction.
  #[must_use]
  fn wrapping_sub(self, rhs: Self) -> Self;
}

impl LinearStorageLen for u8 {
  const BITS: u8 = 5;
  const UPPER_BOUND: Self = u8_cap!();
  const UPPER_BOUND_USIZE: usize = u8_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  type Array = [u8; 1];

  #[inline]
  fn from_le_bytes(array: Self::Array) -> Self {
    u8::from_le_bytes(array)
  }

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn to_le_bytes(self) -> Self::Array {
    self.to_le_bytes()
  }

  #[inline]
  fn usize(self) -> usize {
    self.into()
  }

  #[inline]
  fn wrapping_add(self, rhs: Self) -> Self {
    self.wrapping_add(rhs)
  }

  #[inline]
  fn wrapping_sub(self, rhs: Self) -> Self {
    self.wrapping_sub(rhs)
  }
}

impl LinearStorageLen for u16 {
  const BITS: u8 = 16;
  const UPPER_BOUND: Self = u16_cap!();
  const UPPER_BOUND_USIZE: usize = u16_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  type Array = [u8; 2];

  #[inline]
  fn from_le_bytes(array: Self::Array) -> Self {
    u16::from_le_bytes(array)
  }

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn to_le_bytes(self) -> Self::Array {
    self.to_le_bytes()
  }

  #[inline]
  fn usize(self) -> usize {
    self.into()
  }

  #[inline]
  fn wrapping_add(self, rhs: Self) -> Self {
    self.wrapping_add(rhs)
  }

  #[inline]
  fn wrapping_sub(self, rhs: Self) -> Self {
    self.wrapping_sub(rhs)
  }
}

impl LinearStorageLen for u32 {
  const BITS: u8 = 32;
  const UPPER_BOUND: Self = u32_cap!();
  const UPPER_BOUND_USIZE: usize = u32_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  type Array = [u8; 4];

  #[inline]
  fn from_le_bytes(array: Self::Array) -> Self {
    u32::from_le_bytes(array)
  }

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn to_le_bytes(self) -> Self::Array {
    self.to_le_bytes()
  }

  #[inline]
  fn usize(self) -> usize {
    *Usize::from(self)
  }

  #[inline]
  fn wrapping_add(self, rhs: Self) -> Self {
    self.wrapping_add(rhs)
  }

  #[inline]
  fn wrapping_sub(self, rhs: Self) -> Self {
    self.wrapping_sub(rhs)
  }
}

#[cfg(target_pointer_width = "64")]
impl LinearStorageLen for u64 {
  const BITS: u8 = 64;
  const UPPER_BOUND: Self = u64_cap!();
  const UPPER_BOUND_USIZE: usize = u64_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  type Array = [u8; 8];

  #[inline]
  fn from_le_bytes(array: Self::Array) -> Self {
    u64::from_le_bytes(array)
  }

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn to_le_bytes(self) -> Self::Array {
    self.to_le_bytes()
  }

  #[inline]
  fn usize(self) -> usize {
    *Usize::from(self)
  }

  #[inline]
  fn wrapping_add(self, rhs: Self) -> Self {
    self.wrapping_add(rhs)
  }

  #[inline]
  fn wrapping_sub(self, rhs: Self) -> Self {
    self.wrapping_sub(rhs)
  }
}

impl LinearStorageLen for usize {
  #[expect(clippy::cast_possible_truncation, reason = "lack of const support")]
  const BITS: u8 = usize::BITS as u8;
  const UPPER_BOUND: Self = usize_cap!();
  const UPPER_BOUND_USIZE: usize = usize_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  type Array = [u8; usize::BYTES as usize];

  #[inline]
  fn from_le_bytes(array: Self::Array) -> Self {
    usize::from_le_bytes(array)
  }

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num)
  }

  #[inline]
  fn to_le_bytes(self) -> Self::Array {
    self.to_le_bytes()
  }

  #[inline]
  fn usize(self) -> usize {
    self
  }

  #[inline]
  fn wrapping_add(self, rhs: Self) -> Self {
    self.wrapping_add(rhs)
  }

  #[inline]
  fn wrapping_sub(self, rhs: Self) -> Self {
    self.wrapping_sub(rhs)
  }
}
