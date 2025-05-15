#![expect(
  clippy::cast_possible_truncation,
  reason = "some platforms were removed to allow infallible casts"
)]

#[cfg(target_pointer_width = "16")]
compile_error!("WTX does not support hardwares with pointer sizes less than 32 bits");

macro_rules! u32_max {
  () => {
    4_294_967_295
  };
}

use core::ops::{Deref, DerefMut};

/// An `usize` that can be infallible converted from an `u32`, which effectively drops the support
/// for 16bit hardware.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Usize(usize);

impl Usize {
  pub(crate) const IS_64: bool = cfg!(target_pointer_width = "64");

  #[cfg(feature = "calendar")]
  pub(crate) const fn from_u16(from: u16) -> Self {
    Self(from as usize)
  }

  pub(crate) const fn from_u32(from: u32) -> Self {
    Self(from as usize)
  }

  #[cfg(target_pointer_width = "64")]
  pub(crate) const fn from_u64(from: u64) -> Self {
    Self(from as usize)
  }

  pub(crate) const fn from_usize(from: usize) -> Self {
    Self(from)
  }

  pub(crate) const fn into_usize(self) -> usize {
    self.0
  }

  #[cfg(feature = "mysql")]
  pub(crate) const fn into_saturating_u32(self) -> u32 {
    if self.0 > u32_max!() {
      return u32_max!();
    }
    self.0 as u32
  }

  pub(crate) const fn into_u32(self) -> Option<u32> {
    if self.0 > u32_max!() {
      return None;
    }
    Some(self.0 as u32)
  }

  pub(crate) const fn into_u64(self) -> u64 {
    self.0 as u64
  }
}

impl Deref for Usize {
  type Target = usize;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Usize {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<u8> for Usize {
  #[inline]
  fn from(from: u8) -> Self {
    Self(from.into())
  }
}

impl From<u16> for Usize {
  #[inline]
  fn from(from: u16) -> Self {
    Self(from.into())
  }
}

impl From<u32> for Usize {
  #[inline]
  fn from(from: u32) -> Self {
    Self::from_u32(from)
  }
}

#[cfg(target_pointer_width = "64")]
impl From<u64> for Usize {
  #[inline]
  fn from(from: u64) -> Self {
    Self::from_u64(from)
  }
}

impl From<usize> for Usize {
  #[inline]
  fn from(from: usize) -> Self {
    Self::from_usize(from)
  }
}

impl From<Usize> for u64 {
  #[inline]
  fn from(from: Usize) -> Self {
    from.into_u64()
  }
}

impl From<Usize> for u128 {
  #[inline]
  fn from(from: Usize) -> Self {
    from.0 as u128
  }
}
