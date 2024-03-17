use core::{cell::RefMut, ops::DerefMut};

/// A handle to a held lock.
pub trait LockGuard<'guard, T>: DerefMut<Target = T> {
  /// Sometimes it is desirable to return a type that differs from the original lock.
  type Mapped<U>: LockGuard<'guard, U>
  where
    U: 'guard;

  /// Makes a new mapped element for a component of the locked data.
  fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U>;
}

impl<'guard, T> LockGuard<'guard, T> for RefMut<'guard, T> {
  type Mapped<U> = RefMut<'guard, U>
  where
    U: 'guard;

  #[inline]
  fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U> {
    RefMut::map(this, f)
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
