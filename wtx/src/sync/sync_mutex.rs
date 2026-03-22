use core::ops::{Deref, DerefMut};

cfg_select! {
  feature = "parking_lot" => {
    type LocalMutex<T> = parking_lot::Mutex<T>;
    type LocalMutexGuard<'any, T> = parking_lot::MutexGuard<'any, T>;
  }
  feature = "std" => {
    type LocalMutex<T> = std::sync::Mutex<T>;
    type LocalMutexGuard<'any, T> = std::sync::MutexGuard<'any, T>;
  }
  _ => {
    type LocalMutex<T> = crate::sync::NoStdMutex<T>;
    type LocalMutexGuard<'any, T> = crate::sync::NoStdMutexGuard<'any, T>;
  }
}

/// A mutual exclusion primitive useful for protecting shared data, this mutex will block threads
/// waiting for the lock to become available.
///
/// Delegates to an inner implementation according to the selected features.
#[derive(Debug)]
pub struct SyncMutex<T>(LocalMutex<T>);

impl<T> SyncMutex<T> {
  /// Creates a new mutex.
  #[inline]
  pub const fn new(elem: T) -> Self {
    Self(LocalMutex::new(elem))
  }

  /// Acquires a mutex, blocking the current thread until it is able to do so.
  #[inline]
  pub fn lock(&self) -> SyncMutexGuard<'_, T> {
    cfg_select! {
      feature = "parking_lot" => SyncMutexGuard(self.0.lock()),
      feature = "std" => {
        #[expect(clippy::unwrap_used, clippy::missing_panics_doc, reason = "poison is ignored")]
        SyncMutexGuard(self.0.lock().unwrap())
      }
      _ => SyncMutexGuard(self.0.lock())
    }
  }

  /// Attempts to acquire the lock, returns `None` if it is being used.
  #[inline]
  pub fn try_lock(&self) -> Option<SyncMutexGuard<'_, T>> {
    cfg_select! {
      feature = "parking_lot" => self.0.try_lock().map(SyncMutexGuard),
      feature = "std" => self.0.try_lock().ok().map(SyncMutexGuard),
      _ => self.0.try_lock().map(SyncMutexGuard)
    }
  }
}

/// An RAII implementation of a “scoped lock” of a mutex. When this structure is dropped
/// (falls out of scope), the lock will be unlocked.
///
/// Delegates to an inner implementation according to the selected features.
#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct SyncMutexGuard<'any, T>(LocalMutexGuard<'any, T>);

impl<T> Deref for SyncMutexGuard<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    &self.0
  }
}

impl<T> DerefMut for SyncMutexGuard<'_, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    &mut self.0
  }
}
