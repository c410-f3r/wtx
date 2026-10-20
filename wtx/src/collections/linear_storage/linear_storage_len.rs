// `Usize` only permits platforms with pointer sizes of 32bits or 64bits.

// Rust requires a minimum pointer size of `u16`, therefore, it is safe to use the full length.
macro_rules! u8_cap {
  () => {
    255
  };
}
// `WTX` requires a minimum pointer size of `u32`, therefore, it is safe to use the full length.
macro_rules! u16_cap {
  () => {
    65_535
  };
}
macro_rules! u32_cap {
  () => {
    cfg_select! {
      target_pointer_width = "64" => 4_294_967_295,
      _ => 2_147_483_647,
    }
  };
}
#[cfg(target_pointer_width = "64")]
macro_rules! u64_cap {
  () => {
    9_223_372_036_854_775_807
  };
}
macro_rules! usize_cap {
  () => {
    cfg_select! {
      target_pointer_width = "64" => 9_223_372_036_854_775_807,
      _ => 2_147_483_647,
    }
  };
}

use crate::misc::{ArithmeticError, LeaseMut, TryArithmetic, Usize, int_conv::u8usize};
use core::fmt::Display;

/// Determines how many elements can be stored in a linear collection.
pub trait LinearStorageLen:
  Copy
  + Default
  + Display
  + Eq
  + From<u8>
  + Ord
  + PartialEq
  + PartialOrd
  + Sized
  + TryArithmetic<Self, Output = Self>
{
  /// The number of bits needed to represent the maximum allowed length.
  const BITS: u8;
  /// The size in bytes used to store this length.
  const BYTES: u8;
  /// Instance that represents the number one.
  const ONE: Self;
  /// The maximum number of allowed elements.
  const UPPER_BOUND: Self;
  /// The maximum number of allowed elements as `usize`.
  const UPPER_BOUND_USIZE: usize;
  /// Instance that represents the number zero.
  const ZERO: Self;

  /// Array of bytes that compose this length
  type Array: Default + LeaseMut<[u8]>;

  /// Returns the memory representation of this integer as a byte array in little-endian byte order.
  fn from_le_bytes(array: Self::Array) -> Self;

  /// Tries to create a new instance from a `usize` primitive.
  fn from_usize(num: usize) -> crate::Result<Self>;

  /// Checked integer addition.
  #[must_use = must_use_itself_doc!()]
  fn checked_add(self, rhs: Self) -> Option<Self>;

  /// Checked integer remainder.
  #[must_use = must_use_itself_doc!()]
  fn checked_rem(self, rhs: Self) -> Option<Self>;

  /// Checked integer subtraction.
  #[must_use = must_use_itself_doc!()]
  fn checked_sub(self, rhs: Self) -> Option<Self>;

  /// Saturating integer addition.
  #[must_use = must_use_itself_doc!()]
  fn saturating_add(self, rhs: Self) -> Self;

  /// Saturating integer subtraction.
  #[must_use = must_use_itself_doc!()]
  fn saturating_sub(self, rhs: Self) -> Self;

  /// Returns the memory representation of this integer as a byte array in little-endian byte order.
  fn to_le_bytes(self) -> Self::Array;

  /// Converts itself into `usize`.
  fn usize(self) -> usize;

  /// Wrapping (modular) addition.
  #[must_use = must_use_itself_doc!()]
  fn wrapping_add(self, rhs: Self) -> Self;

  /// Wrapping (modular) subtraction.
  #[must_use = must_use_itself_doc!()]
  fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_linear_storage_len {
  (
    $ty:ty,
    $bits:expr,
    $upper_bound:expr,
    $array:ty,
    $to_usize:expr $(,)?
  ) => {
    impl LinearStorageLen for $ty {
      const BITS: u8 = $bits;
      const BYTES: u8 = $bits / 8;
      const ONE: Self = 1;
      const UPPER_BOUND: Self = $upper_bound;
      const UPPER_BOUND_USIZE: usize = $upper_bound;
      const ZERO: Self = 0;

      type Array = $array;

      #[inline]
      fn from_le_bytes(array: Self::Array) -> Self {
        <$ty>::from_le_bytes(array)
      }

      #[inline]
      fn from_usize(num: usize) -> crate::Result<Self> {
        Ok(num.try_into()?)
      }

      #[inline]
      fn checked_add(self, rhs: Self) -> Option<Self> {
        self.checked_add(rhs)
      }

      #[inline]
      fn checked_rem(self, rhs: Self) -> Option<Self> {
        self.checked_rem(rhs)
      }

      #[inline]
      fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.checked_sub(rhs)
      }

      #[inline]
      fn saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
      }

      #[inline]
      fn saturating_sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
      }

      #[inline]
      fn to_le_bytes(self) -> Self::Array {
        self.to_le_bytes()
      }

      #[inline]
      fn usize(self) -> usize {
        ($to_usize)(self)
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
  };
}

impl_linear_storage_len!(u8, 8, u8_cap!(), [u8; 1], |n: u8| n.into());
impl_linear_storage_len!(u16, 16, u16_cap!(), [u8; 2], |n: u16| n.into());
impl_linear_storage_len!(u32, 32, u32_cap!(), [u8; 4], |n: u32| *Usize::from(n));
#[cfg(target_pointer_width = "64")]
impl_linear_storage_len!(u64, 64, u64_cap!(), [u8; 8], |n: u64| *Usize::from(n));
impl_linear_storage_len!(
  usize,
  if cfg!(target_pointer_width = "64") { 64 } else { 32 },
  usize_cap!(),
  [u8; u8usize(usize::BYTES)],
  |n: usize| n
);

macro_rules! impl_linear_storage_len_wrapper {
  (
    $name:ident,
    $inner:ty,
    $bits:expr,
    $bytes:expr,
    $upper_bound:expr,
    $array:ty $(,)?
  ) => {
    /// Unsigned integer used to calculate lengths of collections.
    ///
    /// All arithmetic operations can surpass their associated upper bounds so it is up to the caller
    /// to avoid overflows.
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct $name($inner);

    impl $name {
      /// The maximum allowed length for collections using this type.
      pub const MAX: Self = Self($upper_bound);

      /// Creates a new instance from the backing primitive.
      #[inline]
      pub const fn new(value: $inner) -> Self {
        Self(value)
      }

      /// Returns the underlying primitive value.
      #[inline]
      pub const fn get(self) -> $inner {
        self.0
      }
    }

    impl Display for $name {
      #[inline]
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
      }
    }

    impl From<u8> for $name {
      #[inline]
      fn from(from: u8) -> Self {
        Self::new(<$inner>::from(from))
      }
    }

    impl From<$name> for $inner {
      #[inline]
      fn from(from: $name) -> Self {
        from.0
      }
    }

    impl TryArithmetic<$name> for $name {
      type Output = $name;

      #[inline]
      fn try_add(&self, rhs: $name) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_add(rhs.0).ok_or(ArithmeticError::AddOverflow)?))
      }

      #[inline]
      fn try_div(&self, rhs: $name) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_div(rhs.0).ok_or(ArithmeticError::DivOverflow)?))
      }

      #[inline]
      fn try_mul(&self, rhs: $name) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_mul(rhs.0).ok_or(ArithmeticError::MulOverflow)?))
      }

      #[inline]
      fn try_pow_i32(&self, exp: i32) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_pow(exp.try_into()?).ok_or(ArithmeticError::PowI32Overflow)?))
      }

      #[inline]
      fn try_pow_u32(&self, exp: u32) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_pow(exp).ok_or(ArithmeticError::PowU32Overflow)?))
      }

      #[inline]
      fn try_rem(&self, rhs: $name) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_rem(rhs.0).ok_or(ArithmeticError::RemOverflow)?))
      }

      #[inline]
      fn try_sub(&self, rhs: $name) -> crate::Result<Self::Output> {
        Ok(Self(self.0.checked_sub(rhs.0).ok_or(ArithmeticError::SubOverflow)?))
      }
    }

    impl LinearStorageLen for $name {
      const BITS: u8 = $bits;
      const BYTES: u8 = $bytes;
      const ONE: Self = Self(1);
      const UPPER_BOUND: Self = Self($upper_bound);
      const UPPER_BOUND_USIZE: usize = $upper_bound;
      const ZERO: Self = Self(0);

      type Array = $array;

      #[inline]
      fn from_le_bytes(array: Self::Array) -> Self {
        Self(<$inner>::from_le_bytes(array))
      }

      #[inline]
      fn from_usize(num: usize) -> crate::Result<Self> {
        Ok(Self(<$inner>::try_from(num)?))
      }

      #[inline]
      fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_add(rhs.0)?))
      }

      #[inline]
      fn checked_rem(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_rem(rhs.0)?))
      }

      #[inline]
      fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_sub(rhs.0)?))
      }

      #[inline]
      fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
      }

      #[inline]
      fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
      }

      #[inline]
      fn to_le_bytes(self) -> Self::Array {
        self.0.to_le_bytes()
      }

      #[inline]
      fn usize(self) -> usize {
        *Usize::from(self.0)
      }

      #[inline]
      fn wrapping_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
      }

      #[inline]
      fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
      }
    }
  };
}

impl_linear_storage_len_wrapper!(U7Len, u8, 7, 1, 127, [u8; 1]);
impl_linear_storage_len_wrapper!(U15Len, u16, 15, 2, 32_767, [u8; 2]);
impl_linear_storage_len_wrapper!(U31Len, u32, 31, 4, 2_147_483_647, [u8; 4]);
#[cfg(target_pointer_width = "64")]
impl_linear_storage_len_wrapper!(U63Len, u64, 63, 8, 9_223_372_036_854_775_807, [u8; 8]);
