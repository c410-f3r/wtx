use crate::pool_manager::LockGuard;
use core::{
  cell::{RefCell, RefMut},
  future::{poll_fn, Future},
  task::Poll,
};

/// An asynchronous mutual exclusion primitive useful for protecting shared data.
pub trait Lock<T> {
  /// See [LockGuard].
  type Guard<'guard>: LockGuard<'guard, T>
  where
    Self: 'guard;

  /// Generic way to build a lock.
  fn new(resource: T) -> Self;

  /// Locks this element, causing the current task to yield until the lock has been acquired. When
  /// the lock has been acquired, returns a guard.
  fn lock(&self) -> impl Future<Output = Self::Guard<'_>>;
}

impl<T> Lock<T> for RefCell<T> {
  type Guard<'guard> = RefMut<'guard, T>
  where
    Self: 'guard;

  #[inline]
  fn new(resource: T) -> Self {
    RefCell::new(resource)
  }

  #[inline]
  fn lock(&self) -> impl Future<Output = Self::Guard<'_>> {
    poll_fn(
      |_| {
        if let Ok(elem) = self.try_borrow_mut() {
          Poll::Ready(elem)
        } else {
          Poll::Pending
        }
      },
    )
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::pool_manager::Lock;
  use tokio::sync::{Mutex, MutexGuard};

  impl<T> Lock<T> for Mutex<T> {
    type Guard<'guard> = MutexGuard<'guard, T>
    where
      Self: 'guard;

    #[inline]
    fn new(resource: T) -> Self {
      Mutex::new(resource)
    }

    #[inline]
    async fn lock(&self) -> Self::Guard<'_> {
      (*self).lock().await
    }
  }
}
