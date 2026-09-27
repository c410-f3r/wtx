use crate::collections::{ArrayString, LinearStorageLen, Vector};
use alloc::{string::String, vec::Vec};

/// Truncates the storage, delimiting its length by `I`.
pub trait Truncate {
  /// Truncates the storage, delimiting its length by `I`.
  fn truncate(&mut self, input: usize);
}

impl<T> Truncate for &mut T
where
  T: Truncate,
{
  #[inline]
  fn truncate(&mut self, input: usize) {
    (*self).truncate(input);
  }
}

impl<L, const N: usize> Truncate for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(L::from_usize(input).unwrap_or(L::UPPER_BOUND));
  }
}

impl<T> Truncate for Option<T> {
  #[inline]
  fn truncate(&mut self, _: usize) {
    *self = None;
  }
}

impl Truncate for String {
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(input);
  }
}

impl<T> Truncate for Vec<T> {
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(input);
  }
}

impl<L, T> Truncate for Vector<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn truncate(&mut self, input: usize) {
    self.truncate(L::from_usize(input).unwrap_or(L::UPPER_BOUND));
  }
}
