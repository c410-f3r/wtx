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
macro_rules! u32_cap {
  () => {
    if cfg!(target_pointer_width = "64") { 4_294_967_295 } else { 2_147_483_647 }
  };
}
macro_rules! u64_cap {
  () => {
    9_223_372_036_854_775_807
  };
}
macro_rules! usize_cap {
  () => {
    if cfg!(target_pointer_width = "64") { 9_223_372_036_854_775_807 } else { 2_147_483_647 }
  };
}

use crate::misc::Usize;

/// Determines how many elements can be stored in a collection.
pub trait StorageLen: Copy + Eq + From<u8> + Ord + PartialEq + PartialOrd + Sized {
  /// The maximum number of elements.
  const CAPACITY: Self;
  /// The maximum number of elements as `usize`.
  const CAPACITY_USIZE: usize;
  /// Instance that represents the number one.
  const ONE: Self;
  /// Instance that represents the number zero.
  const ZERO: Self;

  /// Wrapping (modular) addition.
  fn checked_sub(self, rhs: Self) -> Option<Self>;

  /// Converts itself into `usize`.
  fn usize(self) -> usize;

  /// Wrapping (modular) addition.
  #[must_use]
  fn wrapping_add(self, rhs: Self) -> Self;
}

impl StorageLen for u8 {
  const CAPACITY: Self = u8_cap!();
  const CAPACITY_USIZE: usize = u8_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

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
}

impl StorageLen for u16 {
  const CAPACITY: Self = u16_cap!();
  const CAPACITY_USIZE: usize = u16_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

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
}

impl StorageLen for u32 {
  const CAPACITY: Self = u32_cap!();
  const CAPACITY_USIZE: usize = u32_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

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
}

#[cfg(target_pointer_width = "64")]
impl StorageLen for u64 {
  const CAPACITY: Self = u64_cap!();
  const CAPACITY_USIZE: usize = u64_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

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
}

impl StorageLen for usize {
  const CAPACITY: Self = usize_cap!();
  const CAPACITY_USIZE: usize = usize_cap!();
  const ONE: Self = 1;
  const ZERO: Self = 0;

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
}
