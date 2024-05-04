use core::ops::DerefMut;

/// A handle to a held lock.
pub trait LockGuard<'guard, T>: DerefMut<Target = T> {
  /// Sometimes it is desirable to return a type that differs from the original lock.
  type Mapped<U>: LockGuard<'guard, U>
  where
    U: 'guard;

  /// Makes a new mapped element for a component of the locked data.
  fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U>;
}

#[cfg(feature = "embassy-sync")]
mod embassy {
  use crate::misc::LockGuard;
  use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::MutexGuard};

  impl<'guard, M, T> LockGuard<'guard, T> for MutexGuard<'guard, M, T>
  where
    M: RawMutex,
  {
    type Mapped<U> = MutexGuard<'guard, M, U>
    where
      U: 'guard;

    #[inline]
    fn map<U>(_: Self, _: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U> {
      unimplemented!("Embassy needs to add support for `map` methods");
    }
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::misc::LockGuard;
  use tokio::sync::{MappedMutexGuard, MutexGuard};

  impl<'guard, T> LockGuard<'guard, T> for MappedMutexGuard<'guard, T> {
    type Mapped<U> = MappedMutexGuard<'guard, U>
    where
      U: 'guard;

    #[inline]
    fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U> {
      MappedMutexGuard::map(this, f)
    }
  }

  impl<'guard, T> LockGuard<'guard, T> for MutexGuard<'guard, T> {
    type Mapped<U> = MappedMutexGuard<'guard, U>
    where
      U: 'guard;

    #[inline]
    fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U> {
      MutexGuard::map(this, f)
    }
  }
}
