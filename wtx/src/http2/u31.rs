use core::cmp::Ordering;

const MASK: u32 = 0b0111_1111_1111_1111_1111_1111_1111_1111;

/// Unsigned integer that occupies 32 bits but the actual values are composed by 31 bits.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct U31(u32);

impl U31 {
  pub(crate) const ZERO: Self = Self(0);
  pub(crate) const ONE: Self = Self(1);
  pub(crate) const TWO: Self = Self(2);
  pub(crate) const MAX: Self = Self(2_147_483_647);

  pub(crate) const fn new(value: u32) -> Self {
    Self(value & MASK)
  }

  pub(crate) const fn is_not_zero(self) -> bool {
    self.0 != 0
  }

  pub(crate) const fn is_zero(self) -> bool {
    self.0 == 0
  }

  pub(crate) const fn to_be_bytes(self) -> [u8; 4] {
    self.0.to_be_bytes()
  }

  pub(crate) const fn u32(self) -> u32 {
    self.0
  }

  pub(crate) const fn wrapping_add(self, other: Self) -> Self {
    Self(self.0.wrapping_add(other.0))
  }
}

impl PartialEq<U31> for u32 {
  #[inline]
  fn eq(&self, other: &U31) -> bool {
    *self == other.0
  }
}

impl PartialEq<u32> for U31 {
  #[inline]
  fn eq(&self, other: &u32) -> bool {
    self.0 == *other
  }
}

impl PartialOrd<U31> for u32 {
  #[inline]
  fn partial_cmp(&self, other: &U31) -> Option<Ordering> {
    self.partial_cmp(&other.0)
  }
}

impl PartialOrd<u32> for U31 {
  #[inline]
  fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
    self.0.partial_cmp(other)
  }
}
