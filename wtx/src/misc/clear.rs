use crate::collection::{ArrayString, ArrayVector};
use alloc::{string::String, vec::Vec};

/// See [`Clear::clear`] for more information.
pub trait Clear {
  /// "Clears" the internal buffer, "erasing" all elements.
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

impl<const N: usize> Clear for ArrayString<N> {
  #[inline]
  fn clear(&mut self) {
    self.clear();
  }
}

impl<T, const N: usize> Clear for ArrayVector<T, N> {
  #[inline]
  fn clear(&mut self) {
    self.clear();
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
