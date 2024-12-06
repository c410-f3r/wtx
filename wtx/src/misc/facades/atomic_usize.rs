use core::ops::{Deref, DerefMut};

/// An integer type which can be safely shared between threads.
#[derive(Debug)]
pub(crate) struct AtomicUsize(
  #[cfg(feature = "portable-atomic")] portable_atomic::AtomicUsize,
  #[cfg(not(feature = "portable-atomic"))] core::sync::atomic::AtomicUsize,
);

impl AtomicUsize {
  #[inline]
  pub(crate) const fn new(data: usize) -> Self {
    Self(
      #[cfg(feature = "portable-atomic")]
      portable_atomic::AtomicUsize::new(data),
      #[cfg(not(feature = "portable-atomic"))]
      core::sync::atomic::AtomicUsize::new(data),
    )
  }
}

impl Deref for AtomicUsize {
  #[cfg(feature = "portable-atomic")]
  type Target = portable_atomic::AtomicUsize;
  #[cfg(not(feature = "portable-atomic"))]
  type Target = core::sync::atomic::AtomicUsize;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for AtomicUsize {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
