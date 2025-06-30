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
macro_rules! _u64_cap {
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

use crate::misc::Usize;

/// Determines how many elements can be stored in a collection.
pub trait IndexedStorageLen:
  Copy + Default + Eq + From<u8> + Ord + PartialEq + PartialOrd + Sized
{
  /// The maximum number of elements.
  const UPPER_BOUND: Self;
  /// The maximum number of elements as `usize`.
  const UPPER_BOUND_USIZE: usize;
  /// Instance that represents the number one.
  const ONE: Self;
  /// Instance that represents the number zero.
  const ZERO: Self;

  /// Tries to create a new instance from a `usize` primitive.
  fn from_usize(num: usize) -> crate::Result<Self>;

  /// Checked integer subtraction.
  fn checked_sub(self, rhs: Self) -> Option<Self>;

  /// Converts itself into `usize`.
  fn usize(self) -> usize;

  /// Wrapping (modular) addition.
  #[must_use]
  fn wrapping_add(self, rhs: Self) -> Self;

  /// Wrapping (modular) subtraction.
  #[must_use]
  fn wrapping_sub(self, rhs: Self) -> Self;
}

impl IndexedStorageLen for u8 {
  const UPPER_BOUND: Self = u8_cap!();
  const UPPER_BOUND_USIZE: usize = u8_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn checked_sub(self, rhs: Self) -> Option<Self> {
    self.checked_sub(rhs)
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

impl IndexedStorageLen for u16 {
  const UPPER_BOUND: Self = u16_cap!();
  const UPPER_BOUND_USIZE: usize = u16_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn checked_sub(self, rhs: Self) -> Option<Self> {
    self.checked_sub(rhs)
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

impl IndexedStorageLen for u32 {
  const UPPER_BOUND: Self = u32_cap!();
  const UPPER_BOUND_USIZE: usize = u32_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn checked_sub(self, rhs: Self) -> Option<Self> {
    self.checked_sub(rhs)
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
impl IndexedStorageLen for u64 {
  const UPPER_BOUND: Self = _u64_cap!();
  const UPPER_BOUND_USIZE: usize = _u64_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num.try_into()?)
  }

  #[inline]
  fn checked_sub(self, rhs: Self) -> Option<Self> {
    self.checked_sub(rhs)
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

impl IndexedStorageLen for usize {
  const UPPER_BOUND: Self = usize_cap!();
  const UPPER_BOUND_USIZE: usize = usize_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

  #[inline]
  fn from_usize(num: usize) -> crate::Result<Self> {
    Ok(num)
  }

  #[inline]
  fn checked_sub(self, rhs: Self) -> Option<Self> {
    self.checked_sub(rhs)
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
