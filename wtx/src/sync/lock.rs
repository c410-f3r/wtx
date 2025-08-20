use crate::sync::Arc;
use alloc::rc::Rc;
use core::{
  cell::{RefCell, RefMut},
  future::poll_fn,
  ops::DerefMut,
  task::Poll,
};

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

impl<T> Lock for RefCell<T> {
  type Guard<'guard>
    = RefMut<'guard, Self::Resource>
  where
    Self: 'guard;
  type Resource = T;

  #[inline]
  fn new(resource: Self::Resource) -> Self {
    RefCell::new(resource)
  }

  #[inline]
  async fn lock(&self) -> Self::Guard<'_> {
    poll_fn(|cx| {
      if let Ok(elem) = self.try_borrow_mut() {
        Poll::Ready(elem)
      } else {
        cx.waker().wake_by_ref();
        Poll::Pending
      }
    })
    .await
  }
}

#[cfg(feature = "parking_lot")]
mod parking_lot {
  use crate::sync::Lock;
  use core::{future::poll_fn, task::Poll};
  use parking_lot::{Mutex, MutexGuard};

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
      poll_fn(|cx| {
        if let Some(elem) = self.try_lock() {
          Poll::Ready(elem)
        } else {
          cx.waker().wake_by_ref();
          Poll::Pending
        }
      })
      .await
    }
  }
}

#[cfg(feature = "std")]
mod std {
  use crate::sync::Lock;
  use core::{future::poll_fn, task::Poll};

  impl<T> Lock for std::sync::Mutex<T> {
    type Guard<'guard>
      = std::sync::MutexGuard<'guard, Self::Resource>
    where
      Self: 'guard;
    type Resource = T;

    #[inline]
    fn new(resource: Self::Resource) -> Self {
      std::sync::Mutex::new(resource)
    }

    #[inline]
    async fn lock(&self) -> Self::Guard<'_> {
      poll_fn(|cx| {
        if let Ok(elem) = self.try_lock() {
          Poll::Ready(elem)
        } else {
          cx.waker().wake_by_ref();
          Poll::Pending
        }
      })
      .await
    }
  }

  impl<T> Lock for crate::sync::Mutex<T> {
    type Guard<'guard>
      = crate::sync::MutexGuard<'guard, Self::Resource>
    where
      Self: 'guard;
    type Resource = T;

    #[inline]
    fn new(resource: Self::Resource) -> Self {
      crate::sync::Mutex::new(resource)
    }

    #[inline]
    async fn lock(&self) -> Self::Guard<'_> {
      let rslt = (*self).lock().await;
      // SAFETY: Future is not polled again after finalization
      unsafe { rslt.unwrap_unchecked() }
    }
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::sync::Lock;
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
