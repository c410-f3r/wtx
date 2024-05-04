/// Copy of [core::borrow::Borrow] used to workaround orphan rules.
pub trait Lease<T>
where
  T: ?Sized,
{
  /// Immutable borrow.
  fn lease(&self) -> &T;
}

impl<T, U> Lease<U> for &T
where
  T: Lease<U> + ?Sized,
  U: ?Sized,
{
  #[inline]
  fn lease(&self) -> &U {
    <T as Lease<U>>::lease(*self)
  }
}

impl<T, U> Lease<U> for &mut T
where
  T: Lease<U> + ?Sized,
  U: ?Sized,
{
  #[inline]
  fn lease(&self) -> &U {
    <T as Lease<U>>::lease(*self)
  }
}

/// Copy of [core::borrow::BorrowMut] used to workaround orphan rules.
pub trait LeaseMut<T>: Lease<T>
where
  T: ?Sized,
{
  /// Mutable borrow.
  fn lease_mut(&mut self) -> &mut T;
}

impl<T, U> LeaseMut<U> for &mut T
where
  T: LeaseMut<U> + ?Sized,
  U: ?Sized,
{
  #[inline]
  fn lease_mut(&mut self) -> &mut U {
    <T as LeaseMut<U>>::lease_mut(*self)
  }
}

mod collections {
  use crate::misc::{Lease, LeaseMut};
  use alloc::vec::Vec;

  impl<T> Lease<[T]> for [T] {
    #[inline]
    fn lease(&self) -> &[T] {
      self
    }
  }

  impl<T, const N: usize> Lease<[T]> for [T; N] {
    #[inline]
    fn lease(&self) -> &[T] {
      self
    }
  }

  impl<T> Lease<[T]> for Vec<T> {
    #[inline]
    fn lease(&self) -> &[T] {
      self
    }
  }

  impl<T> Lease<Vec<T>> for Vec<T> {
    #[inline]
    fn lease(&self) -> &Vec<T> {
      self
    }
  }

  impl<T> LeaseMut<[T]> for [T] {
    #[inline]
    fn lease_mut(&mut self) -> &mut [T] {
      self
    }
  }

  impl<T, const N: usize> LeaseMut<[T]> for [T; N] {
    #[inline]
    fn lease_mut(&mut self) -> &mut [T] {
      self
    }
  }

  impl<T> LeaseMut<[T]> for Vec<T> {
    #[inline]
    fn lease_mut(&mut self) -> &mut [T] {
      self
    }
  }

  impl<T> LeaseMut<Vec<T>> for Vec<T> {
    #[inline]
    fn lease_mut(&mut self) -> &mut Vec<T> {
      self
    }
  }
}

#[cfg(feature = "ring")]
mod ring {
  use crate::misc::Lease;
  use ring::digest::Digest;

  impl Lease<[u8]> for Digest {
    #[inline]
    fn lease(&self) -> &[u8] {
      self.as_ref()
    }
  }
}

mod smart_pointers {
  use crate::misc::{Lease, LeaseMut};
  use alloc::{boxed::Box, sync::Arc};

  impl<T> Lease<T> for Arc<T> {
    #[inline]
    fn lease(&self) -> &T {
      self
    }
  }

  impl<T> Lease<T> for Box<T> {
    #[inline]
    fn lease(&self) -> &T {
      self
    }
  }

  impl<T> LeaseMut<T> for Box<T> {
    #[inline]
    fn lease_mut(&mut self) -> &mut T {
      self
    }
  }
}

mod str {
  use crate::misc::Lease;
  use alloc::string::String;

  impl Lease<[u8]> for str {
    #[inline]
    fn lease(&self) -> &[u8] {
      self.as_bytes()
    }
  }

  impl Lease<str> for str {
    #[inline]
    fn lease(&self) -> &str {
      self
    }
  }

  impl Lease<[u8]> for String {
    #[inline]
    fn lease(&self) -> &[u8] {
      self.as_bytes()
    }
  }

  impl Lease<str> for String {
    #[inline]
    fn lease(&self) -> &str {
      self
    }
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::misc::{Lease, LeaseMut};
  use tokio::sync::{MappedMutexGuard, MutexGuard};

  impl<T> Lease<T> for MappedMutexGuard<'_, T> {
    #[inline]
    fn lease(&self) -> &T {
      self
    }
  }

  impl<T> LeaseMut<T> for MappedMutexGuard<'_, T> {
    #[inline]
    fn lease_mut(&mut self) -> &mut T {
      self
    }
  }

  impl<T> Lease<T> for MutexGuard<'_, T> {
    #[inline]
    fn lease(&self) -> &T {
      self
    }
  }

  impl<T> LeaseMut<T> for MutexGuard<'_, T> {
    #[inline]
    fn lease_mut(&mut self) -> &mut T {
      self
    }
  }
}
