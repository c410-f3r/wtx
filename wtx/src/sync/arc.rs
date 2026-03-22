#![allow(clippy::disallowed_types, reason = "This is the only allowed place")]

use alloc::boxed::Box;
use core::ops::Deref;

cfg_select! {
  feature = "portable-atomic-util" => {
    type LocalTy<T> = portable_atomic_util::Arc<T>;
  }
  _ => {
    type LocalTy<T> = alloc::sync::Arc<T>;
  }
}

/// A thread-safe reference-counting pointer. ‘Arc’ stands for ‘Atomically Reference Counted’.
#[derive(Debug)]
pub struct Arc<T>(LocalTy<T>)
where
  T: ?Sized;

impl<T> Arc<T> {
  /// Constructs a new instance.
  #[inline]
  pub fn new(data: T) -> Self {
    Self(LocalTy::new(data))
  }

  /// Returns a mutable reference into the given `Arc`, if there are
  /// no other `Arc` or `Weak` pointers to the same allocation.
  #[inline]
  pub fn get_mut(this: &mut Self) -> Option<&mut T> {
    LocalTy::get_mut(&mut this.0)
  }

  /// Returns the inner value, if the Arc has exactly one strong reference.
  #[inline]
  pub fn into_inner(this: Self) -> Option<T> {
    LocalTy::into_inner(this.0)
  }

  /// Gets the number of strong pointers to this allocation.
  #[inline]
  pub fn strong_count(this: &Self) -> usize {
    LocalTy::strong_count(&this.0)
  }
}

impl<T> Clone for Arc<T>
where
  T: ?Sized,
{
  #[inline]
  fn clone(&self) -> Self {
    Self(LocalTy::clone(&self.0))
  }
}

impl<T> Deref for Arc<T>
where
  T: ?Sized,
{
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> Eq for Arc<T> where T: Eq + ?Sized {}

impl<T> From<Box<T>> for Arc<T>
where
  T: ?Sized,
{
  #[inline]
  fn from(value: Box<T>) -> Self {
    Self(LocalTy::from(value))
  }
}

impl<T> PartialEq for Arc<T>
where
  T: PartialEq + ?Sized,
{
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.0.eq(&other.0)
  }
}

cfg_select! {
  feature = "portable-atomic-util" => {
    impl<T> From<Arc<T>> for portable_atomic_util::Arc<T> {
      #[inline]
      fn from(from: Arc<T>) -> Self {
        from.0
      }
    }

    impl<T> From<portable_atomic_util::Arc<T>> for Arc<T> {
      #[inline]
      fn from(from: portable_atomic_util::Arc<T>) -> Self {
        Self(from)
      }
    }
  }
  _ => {
    impl<T> From<Arc<T>> for alloc::sync::Arc<T> {
      #[inline]
      fn from(from: Arc<T>) -> Self {
        from.0
      }
    }

    impl<T> From<alloc::sync::Arc<T>> for Arc<T> {
      #[inline]
      fn from(from: alloc::sync::Arc<T>) -> Self {
        Self(from)
      }
    }
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
