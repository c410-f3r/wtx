use crate::collection::{ArrayString, ArrayVector, LinearStorageLen, Vector};
use alloc::{string::String, vec::Vec};

/// See [`Clear::clear`] for more information.
pub trait Clear {
  /// Clears the instance, removing all values.
  fn clear(&mut self);
}

impl<T> Clear for &mut T
where
  T: Clear,
{
  #[inline]
  fn clear(&mut self) {
    (*self).clear();
  }
}

impl Clear for () {
  #[inline]
  fn clear(&mut self) {}
}

impl<L, const N: usize> Clear for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn clear(&mut self) {
    self.truncate(L::ZERO);
  }
}

impl<L, T, const N: usize> Clear for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn clear(&mut self) {
    self.truncate(L::ZERO);
  }
}

impl<T> Clear for Option<T> {
  #[inline]
  fn clear(&mut self) {
    *self = None;
  }
}

impl Clear for String {
  #[inline]
  fn clear(&mut self) {
    self.clear();
  }
}

impl<T> Clear for Vec<T> {
  #[inline]
  fn clear(&mut self) {
    self.clear();
  }
}

impl<T> Clear for Vector<T> {
  #[inline]
  fn clear(&mut self) {
    self.truncate(0);
  }
}
