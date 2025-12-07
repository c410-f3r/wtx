use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub(crate) struct NoStdMutex<T>(T);

impl<T> NoStdMutex<T> {
  pub(crate) const fn new(elem: T) -> Self {
    Self(elem)
  }

  pub(crate) fn lock(&self) -> NoStdMutexGuard<'_, T> {
    NoStdMutexGuard(&self.0)
  }

  pub(crate) fn try_lock(&self) -> Option<NoStdMutexGuard<'_, T>> {
    None
  }
}

#[clippy::has_significant_drop]
#[derive(Debug)]
pub(crate) struct NoStdMutexGuard<'any, T>(&'any T);

impl<T> Deref for NoStdMutexGuard<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    &self.0
  }
}

impl<T> DerefMut for NoStdMutexGuard<'_, T> {
  #[expect(clippy::panic, reason = "not implemented yet")]
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    panic!("Operation not supported yet");
  }
}
