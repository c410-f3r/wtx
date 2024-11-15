use core::ops::Deref;

/// A thread-safe reference-counting pointer. ‘Arc’ stands for ‘Atomically Reference Counted’.
#[derive(Debug)]
pub struct Arc<T>(
  #[cfg(feature = "portable-atomic-util")] portable_atomic_util::Arc<T>,
  #[cfg(not(feature = "portable-atomic-util"))] alloc::sync::Arc<T>,
);

impl<T> Arc<T> {
  #[inline]
  pub(crate) fn new(data: T) -> Self {
    Self(
      #[cfg(feature = "portable-atomic-util")]
      portable_atomic_util::Arc::new(data),
      #[cfg(not(feature = "portable-atomic-util"))]
      alloc::sync::Arc::new(data),
    )
  }
}

impl<T> Clone for Arc<T> {
  #[inline]
  fn clone(&self) -> Self {
    #[cfg(feature = "portable-atomic-util")]
    let data = portable_atomic_util::Arc::clone(&self.0);
    #[cfg(not(feature = "portable-atomic-util"))]
    let data = alloc::sync::Arc::clone(&self.0);
    Self(data)
  }
}

impl<T> Deref for Arc<T> {
  #[cfg(feature = "portable-atomic-util")]
  type Target = T;
  #[cfg(not(feature = "portable-atomic-util"))]
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(not(feature = "portable-atomic-util"))]
impl<T> From<Arc<T>> for alloc::sync::Arc<T> {
  #[inline]
  fn from(from: Arc<T>) -> Self {
    from.0
  }
}

#[cfg(feature = "portable-atomic-util")]
impl<T> From<Arc<T>> for portable_atomic_util::Arc<T> {
  #[inline]
  fn from(from: Arc<T>) -> Self {
    from.0
  }
}
