use crate::sync::Arc;
use alloc::rc::Rc;
use core::ops::DerefMut;

/// An asynchronous mutual exclusion primitive useful for protecting shared data.
pub trait Lock {
  /// Lock type.
  type Guard<'guard>: DerefMut<Target = Self::Resource>
  where
    Self: 'guard;
  /// Resource behind the lock.
  type Resource;

  /// Generic way to build a lock.
  fn new(resource: Self::Resource) -> Self;

  /// Locks this element, causing the current task to yield until the lock has been acquired. When
  /// the lock has been acquired, returns a guard.
  fn lock(&self) -> impl Future<Output = Self::Guard<'_>>;
}

impl<T> Lock for Arc<T>
where
  T: Lock,
{
  type Guard<'guard>
    = T::Guard<'guard>
  where
    Self: 'guard;
  type Resource = T::Resource;

  #[inline]
  fn new(resource: Self::Resource) -> Self {
    Arc::new(T::new(resource))
  }

  #[inline]
  async fn lock(&self) -> Self::Guard<'_> {
    (**self).lock().await
  }
}

impl<T> Lock for Rc<T>
where
  T: Lock,
{
  type Guard<'guard>
    = T::Guard<'guard>
  where
    Self: 'guard;
  type Resource = T::Resource;

  #[inline]
  fn new(resource: Self::Resource) -> Self {
    Rc::new(T::new(resource))
  }

  #[inline]
  async fn lock(&self) -> Self::Guard<'_> {
    (**self).lock().await
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::misc::Lock;
  use tokio::sync::{Mutex, MutexGuard};

  impl<T> Lock for Mutex<T> {
    type Guard<'guard>
      = MutexGuard<'guard, Self::Resource>
    where
      Self: 'guard;
    type Resource = T;

    #[inline]
    fn new(resource: Self::Resource) -> Self {
      Mutex::new(resource)
    }

    #[inline]
    async fn lock(&self) -> Self::Guard<'_> {
      (*self).lock().await
    }
  }
}
