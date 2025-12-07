#[cfg(not(any(feature = "loom", feature = "parking_lot", feature = "std")))]
mod no_std;

use core::ops::{Deref, DerefMut};

#[cfg(feature = "parking_lot")]
type LocalMutex<T> = parking_lot::Mutex<T>;
#[cfg(all(feature = "loom", not(any(feature = "parking_lot"))))]
type LocalMutex<T> = loom::sync::Mutex<T>;
#[cfg(all(feature = "std", not(any(feature = "loom", feature = "parking_lot"))))]
type LocalMutex<T> = std::sync::Mutex<T>;
#[cfg(not(any(feature = "loom", feature = "parking_lot", feature = "std")))]
type LocalMutex<T> = no_std::NoStdMutex<T>;

#[cfg(feature = "parking_lot")]
type LocalMutexGuard<'any, T> = parking_lot::MutexGuard<'any, T>;
#[cfg(all(feature = "loom", not(any(feature = "parking_lot"))))]
type LocalMutexGuard<'any, T> = loom::sync::MutexGuard<'any, T>;
#[cfg(all(feature = "std", not(any(feature = "loom", feature = "parking_lot"))))]
type LocalMutexGuard<'any, T> = std::sync::MutexGuard<'any, T>;
#[cfg(not(any(feature = "loom", feature = "parking_lot", feature = "std")))]
type LocalMutexGuard<'any, T> = no_std::NoStdMutexGuard<'any, T>;

/// A mutual exclusion primitive useful for protecting shared data.
#[derive(Debug)]
pub struct SyncMutex<T>(LocalMutex<T>);

impl<T> SyncMutex<T> {
  /// Creates a new mutex.
  #[cfg(feature = "loom")]
  #[inline]
  pub fn new(elem: T) -> Self {
    Self(LocalMutex::new(elem))
  }
  /// Creates a new mutex.
  #[cfg(not(feature = "loom"))]
  #[inline]
  pub const fn new(elem: T) -> Self {
    Self(LocalMutex::new(elem))
  }

  /// Acquires a mutex, blocking the current thread until it is able to do so.
  #[inline]
  pub fn lock(&self) -> SyncMutexGuard<'_, T> {
    #[cfg(any(
      feature = "parking_lot",
      not(any(feature = "loom", feature = "parking_lot", feature = "std"))
    ))]
    return SyncMutexGuard(self.0.lock());
    #[cfg(not(any(
      feature = "parking_lot",
      not(any(feature = "loom", feature = "parking_lot", feature = "std"))
    )))]
    #[expect(clippy::unwrap_used, clippy::missing_panics_doc, reason = "poison is ignored")]
    return SyncMutexGuard(self.0.lock().unwrap());
  }

  /// Attempts to acquire this lock.
  #[inline]
  pub fn try_lock(&self) -> Option<SyncMutexGuard<'_, T>> {
    #[cfg(any(
      feature = "parking_lot",
      not(any(feature = "loom", feature = "parking_lot", feature = "std"))
    ))]
    return self.0.try_lock().map(SyncMutexGuard);
    #[cfg(not(any(
      feature = "parking_lot",
      not(any(feature = "loom", feature = "parking_lot", feature = "std"))
    )))]
    return self.0.try_lock().ok().map(SyncMutexGuard);
  }
}

/// A RAII implementation of a "scoped lock" of a mutex.
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
