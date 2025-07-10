#![allow(clippy::disallowed_types, reason = "This is the only allowed place")]

use core::ops::Deref;

/// A thread-safe reference-counting pointer. ‘Arc’ stands for ‘Atomically Reference Counted’.
#[derive(Debug)]
pub struct Arc<T>(
  #[cfg(feature = "portable-atomic-util")] portable_atomic_util::Arc<T>,
  #[cfg(not(feature = "portable-atomic-util"))] alloc::sync::Arc<T>,
);

impl<T> Arc<T> {
  /// Constructs a new instance.
  #[inline]
  pub fn new(data: T) -> Self {
    Self(
      #[cfg(feature = "portable-atomic-util")]
      portable_atomic_util::Arc::new(data),
      #[cfg(not(feature = "portable-atomic-util"))]
      alloc::sync::Arc::new(data),
    )
  }

  /// Returns a mutable reference into the given `Arc`, if there are
  /// no other `Arc` or `Weak` pointers to the same allocation.
  #[inline]
  pub fn get_mut(this: &mut Self) -> Option<&mut T> {
    #[cfg(feature = "portable-atomic-util")]
    return portable_atomic_util::Arc::get_mut(&mut this.0);
    #[cfg(not(feature = "portable-atomic-util"))]
    return alloc::sync::Arc::get_mut(&mut this.0);
  }

  /// Returns the inner value, if the Arc has exactly one strong reference.
  #[inline]
  pub fn into_inner(this: Self) -> Option<T> {
    #[cfg(feature = "portable-atomic-util")]
    return portable_atomic_util::Arc::into_inner(this.0);
    #[cfg(not(feature = "portable-atomic-util"))]
    return alloc::sync::Arc::into_inner(this.0);
  }

  /// Gets the number of strong pointers to this allocation.
  #[inline]
  pub fn strong_count(this: &Self) -> usize {
    #[cfg(feature = "portable-atomic-util")]
    return portable_atomic_util::Arc::strong_count(&this.0);
    #[cfg(not(feature = "portable-atomic-util"))]
    return alloc::sync::Arc::strong_count(&this.0);
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
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(feature = "portable-atomic-util")]
impl<T> From<Arc<T>> for portable_atomic_util::Arc<T> {
  #[inline]
  fn from(from: Arc<T>) -> Self {
    from.0
  }
}

#[cfg(feature = "portable-atomic-util")]
impl<T> From<portable_atomic_util::Arc<T>> for Arc<T> {
  #[inline]
  fn from(from: portable_atomic_util::Arc<T>) -> Self {
    Self(from)
  }
}

#[cfg(not(feature = "portable-atomic-util"))]
impl<T> From<Arc<T>> for alloc::sync::Arc<T> {
  #[inline]
  fn from(from: Arc<T>) -> Self {
    from.0
  }
}

#[cfg(not(feature = "portable-atomic-util"))]
impl<T> From<alloc::sync::Arc<T>> for Arc<T> {
  #[inline]
  fn from(from: alloc::sync::Arc<T>) -> Self {
    Self(from)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::sync::Arc;
  use serde::{Serialize, Serializer};

  impl<T> Serialize for Arc<T>
  where
    T: Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      T::serialize(self, serializer)
    }
  }
}
