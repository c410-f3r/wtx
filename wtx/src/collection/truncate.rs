use crate::collection::{ArrayString, ArrayVector, LinearStorageLen, Vector};
use alloc::{string::String, vec::Vec};

/// Truncates the storage, delimiting its length by `I`.
pub trait Truncate<I> {
  /// Truncates the storage, delimiting its length by `I`.
  fn truncate(&mut self, input: I);
}

impl<T, I> Truncate<I> for &mut T
where
  T: Truncate<I>,
{
  #[inline]
  fn truncate(&mut self, input: I) {
    (*self).truncate(input);
  }
}

impl<L, const N: usize> Truncate<L> for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn truncate(&mut self, input: L) {
    self.truncate(input);
  }
}

impl<L, T, const N: usize> Truncate<L> for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn truncate(&mut self, input: L) {
    self.truncate(input);
  }
}

impl<T, I> Truncate<I> for Option<T> {
  #[inline]
  fn truncate(&mut self, _: I) {
    *self = None;
  }
}

impl Truncate<usize> for String {
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(input);
  }
}

impl<T> Truncate<usize> for Vec<T> {
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(input);
  }
}

impl<T> Truncate<usize> for Vector<T> {
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(input);
  }
}
