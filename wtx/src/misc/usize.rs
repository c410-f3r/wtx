#![allow(
  // Some platforms were removed to allow infallible casts
  clippy::as_conversions
)]

use core::ops::{Deref, DerefMut};

/// An `usize` that can be infallible converted from an `u32`. In other words, this effectively kills
/// the support for 16bit hardware.
///
/// Additionally, 128bit support is also dropped.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Usize(usize);

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
    #[cfg(target_pointer_width = "16")]
    compile_error!("WTX does not support 16bit hardware");
    Self(from as usize)
  }
}

impl From<usize> for Usize {
  #[inline]
  fn from(from: usize) -> Self {
    Self(from)
  }
}

impl From<Usize> for u64 {
  #[inline]
  fn from(from: Usize) -> Self {
    #[cfg(target_pointer_width = "128")]
    compile_error!("WTX does not support 128bit hardware");
    from.0 as u64
  }
}

impl From<Usize> for u128 {
  #[inline]
  fn from(from: Usize) -> Self {
    from.0 as u128
  }
}
