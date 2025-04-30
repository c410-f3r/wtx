use core::ops::{Deref, DerefMut};

/// Prevents false sharing by padding and aligning to the length of a cache line.
#[cfg_attr(
  any(target_arch = "aarch64", target_arch = "powerpc64", target_arch = "x86_64",),
  repr(align(128))
)]
#[cfg_attr(
  not(any(target_arch = "aarch64", target_arch = "powerpc64", target_arch = "x86_64",)),
  repr(align(64))
)]
#[derive(Debug)]
pub struct CachePadded<T>(
  /// Element
  pub T,
);

impl<T> Deref for CachePadded<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    &self.0
  }
}

impl<T> DerefMut for CachePadded<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    &mut self.0
  }
}

impl<T> From<T> for CachePadded<T> {
  #[inline]
  fn from(t: T) -> Self {
    CachePadded(t)
  }
}
