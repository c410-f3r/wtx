use crate::sync::Backoff;
use core::{
  cell::UnsafeCell,
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicBool, Ordering},
};

/// A mutual exclusion primitive useful for protecting shared data.
///
/// Utilizes a spinlock to allow usage in `no_std` environments.
pub struct NoStdMutex<T> {
  data: UnsafeCell<T>,
  is_locked: AtomicBool,
}

impl<T> NoStdMutex<T> {
  /// New instance
  #[inline]
  pub const fn new(elem: T) -> Self {
    Self { data: UnsafeCell::new(elem), is_locked: AtomicBool::new(false) }
  }

  /// Spins until it acquires the lock.
  #[inline]
  pub fn lock(&self) -> NoStdMutexGuard<'_, T> {
    let backoff = Backoff::new();
    loop {
      let is_free = self
        .is_locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok();
      if is_free {
        return NoStdMutexGuard(self);
      }
      while self.is_locked.load(Ordering::Relaxed) {
        backoff.snooze();
      }
    }
  }

  /// Attempts to acquire the lock, returns `None` if it is being used.
  #[inline]
  pub fn try_lock(&self) -> Option<NoStdMutexGuard<'_, T>> {
    self
      .is_locked
      .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
      .ok()
      .map(|_| NoStdMutexGuard(self))
  }
}

impl<T: Debug> Debug for NoStdMutex<T> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NoStdMutex").finish()
  }
}

// SAFETY: The spin-lock guarantees synchronized access to `T`.
unsafe impl<T: Send> Send for NoStdMutex<T> {}
// SAFETY: The spin-lock guarantees synchronized access to `T`.
unsafe impl<T: Send> Sync for NoStdMutex<T> {}

/// An RAII implementation of a “scoped lock” of a mutex.
#[clippy::has_significant_drop]
pub struct NoStdMutexGuard<'any, T>(&'any NoStdMutex<T>);

impl<T: Debug> Debug for NoStdMutexGuard<'_, T> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    (**self).fmt(f)
  }
}

impl<T> Deref for NoStdMutexGuard<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    // SAFETY: Exclusive access because the lock is held by the struct.
    unsafe { &*self.0.data.get() }
  }
}

impl<T> DerefMut for NoStdMutexGuard<'_, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    // SAFETY: Exclusive access because the lock is held by the struct.
    unsafe { &mut *self.0.data.get() }
  }
}

impl<T> Drop for NoStdMutexGuard<'_, T> {
  #[inline]
  fn drop(&mut self) {
    self.0.is_locked.store(false, Ordering::Release);
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    sync::{Arc, NoStdMutex},
  };
  use std::thread;

  #[test]
  fn lock() {
    let mutex = NoStdMutex::new(42);
    let guard = mutex.lock();
    assert_eq!(*guard, 42);
  }

  #[test]
  fn concurrent_readers_and_writers() {
    let mutex = Arc::new(NoStdMutex::new(0i64));
    let num_readers = 4;
    let num_writers = 4;
    let ops_per_thread = 5_000;
    let mut handles = Vector::new();

    for _ in 0..num_writers {
      let local_mutex = Arc::clone(&mutex);
      handles
        .push(thread::spawn(move || {
          for _ in 0..ops_per_thread {
            *local_mutex.lock() += 1;
          }
        }))
        .unwrap();
    }

    for _ in 0..num_readers {
      let local_mutex = Arc::clone(&mutex);
      handles
        .push(thread::spawn(move || {
          for _ in 0..ops_per_thread {
            assert!(*local_mutex.lock() >= 0);
          }
        }))
        .unwrap();
    }

    for handle in handles {
      handle.join().unwrap();
    }

    assert_eq!(*mutex.lock(), num_writers * ops_per_thread);
  }
}
