use core::{cell::RefMut, ops::DerefMut};

/// A handle to a held lock.
pub trait LockGuard<'guard, T>: DerefMut<Target = T>
where
  T: ?Sized,
{
  /// Sometimes it is desirable to return a type that differs from the original lock.
  type Mapped<U>
  where
    U: ?Sized + 'guard;

  /// Makes a new mapped element for a component of the locked data.
  fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U>
  where
    U: ?Sized;
}

impl<'guard, T> LockGuard<'guard, T> for RefMut<'guard, T>
where
  T: ?Sized,
{
  type Mapped<U> = RefMut<'guard, U>
  where
    U: ?Sized + 'guard;

  #[inline]
  fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U>
  where
    U: ?Sized,
  {
    RefMut::map(this, f)
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::pool_manager::LockGuard;
  use tokio::sync::{MappedMutexGuard, MutexGuard};

  impl<'guard, T> LockGuard<'guard, T> for MutexGuard<'guard, T>
  where
    T: ?Sized,
  {
    type Mapped<U> = MappedMutexGuard<'guard, U>
    where
      U: ?Sized + 'guard;

    #[inline]
    fn map<U>(this: Self, f: impl FnOnce(&mut T) -> &mut U) -> Self::Mapped<U>
    where
      U: ?Sized,
    {
      MutexGuard::map(this, f)
    }
  }
}
