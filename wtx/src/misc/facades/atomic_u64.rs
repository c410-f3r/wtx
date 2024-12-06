use core::ops::{Deref, DerefMut};

/// An integer type which can be safely shared between threads.
#[derive(Debug)]
pub(crate) struct AtomicU64(
  #[cfg(feature = "portable-atomic")] portable_atomic::AtomicU64,
  #[cfg(not(feature = "portable-atomic"))] core::sync::atomic::AtomicU64,
);

impl AtomicU64 {
  #[inline]
  pub(crate) const fn new(data: u64) -> Self {
    Self(
      #[cfg(feature = "portable-atomic")]
      portable_atomic::AtomicU64::new(data),
      #[cfg(not(feature = "portable-atomic"))]
      core::sync::atomic::AtomicU64::new(data),
    )
  }
}

impl Deref for AtomicU64 {
  #[cfg(feature = "portable-atomic")]
  type Target = portable_atomic::AtomicU64;
  #[cfg(not(feature = "portable-atomic"))]
  type Target = core::sync::atomic::AtomicU64;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for AtomicU64 {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
