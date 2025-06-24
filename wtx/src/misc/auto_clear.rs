use crate::misc::Clear;
use core::{
  borrow::{Borrow, BorrowMut},
  ops::{Deref, DerefMut},
};

/// Any mutable item wrapped in this structure is automatically cleaned when dropped.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AutoClear<T>(T)
where
  T: Clear;

impl<T> AsMut<T> for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn as_mut(&mut self) -> &mut T {
    self
  }
}

impl<T> AsRef<T> for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn as_ref(&self) -> &T {
    self
  }
}

impl<T> Borrow<T> for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn borrow(&self) -> &T {
    self
  }
}

impl<T> BorrowMut<T> for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn borrow_mut(&mut self) -> &mut T {
    self
  }
}

impl<T> Deref for AutoClear<T>
where
  T: Clear,
{
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> Drop for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn drop(&mut self) {
    self.0.clear();
  }
}

impl<T> From<T> for AutoClear<T>
where
  T: Clear,
{
  #[inline]
  fn from(from: T) -> Self {
    Self(from)
  }
}
