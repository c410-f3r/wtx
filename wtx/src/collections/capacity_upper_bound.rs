use crate::collections::{ArrayString, ArrayVector, LinearStorageLen, Vector};
use alloc::{string::String, vec::Vec};

/// The maximum theoretical number of elements a type implementation is able to store.
pub trait CapacityUpperBound {
  /// The maximum theoretical number of elements a type implementation is able to store.
  const CAPACITY_UPPER_BOUND: usize;

  /// Instance method representing [`Self::CAPACITY_UPPER_BOUND`].
  #[inline]
  fn capacity_upper_bound(&self) -> usize {
    Self::CAPACITY_UPPER_BOUND
  }
}

impl<T> CapacityUpperBound for &T
where
  T: CapacityUpperBound,
{
  const CAPACITY_UPPER_BOUND: usize = T::CAPACITY_UPPER_BOUND;

  #[inline]
  fn capacity_upper_bound(&self) -> usize {
    (*self).capacity_upper_bound()
  }
}

impl CapacityUpperBound for () {
  const CAPACITY_UPPER_BOUND: usize = 0;
}

impl<T> CapacityUpperBound for Option<T> {
  const CAPACITY_UPPER_BOUND: usize = 1;
}

impl<T, const N: usize> CapacityUpperBound for [T; N] {
  const CAPACITY_UPPER_BOUND: usize = N;
}

impl<T> CapacityUpperBound for &'_ [T] {
  const CAPACITY_UPPER_BOUND: usize = isize::MAX.unsigned_abs();
}

impl<T> CapacityUpperBound for &'_ mut [T] {
  const CAPACITY_UPPER_BOUND: usize = isize::MAX.unsigned_abs();
}

impl<L, T, const N: usize> CapacityUpperBound for ArrayVector<L, T, N>
where
  L: LinearStorageLen,
{
  const CAPACITY_UPPER_BOUND: usize = N;
}

impl<L, const N: usize> CapacityUpperBound for ArrayString<L, N>
where
  L: LinearStorageLen,
{
  const CAPACITY_UPPER_BOUND: usize = N;
}

impl CapacityUpperBound for String {
  const CAPACITY_UPPER_BOUND: usize = isize::MAX.unsigned_abs();
}

impl<T> CapacityUpperBound for Vec<T> {
  const CAPACITY_UPPER_BOUND: usize = isize::MAX.unsigned_abs();
}

impl<L, T> CapacityUpperBound for Vector<L, T>
where
  L: LinearStorageLen,
{
  const CAPACITY_UPPER_BOUND: usize = L::UPPER_BOUND_USIZE;
}
