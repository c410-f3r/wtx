use crate::collection::{ArrayString, ArrayVector, LinearStorageLen, Vector};
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
  const CAPACITY_UPPER_BOUND: usize = capacity_upper_bound_of_type::<T>();
}

impl<T> CapacityUpperBound for &'_ mut [T] {
  const CAPACITY_UPPER_BOUND: usize = capacity_upper_bound_of_type::<T>();
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
  const CAPACITY_UPPER_BOUND: usize = capacity_upper_bound_of_type::<u8>();
}

impl<T> CapacityUpperBound for Vec<T> {
  const CAPACITY_UPPER_BOUND: usize = capacity_upper_bound_of_type::<T>();
}

impl<T> CapacityUpperBound for Vector<T> {
  const CAPACITY_UPPER_BOUND: usize = capacity_upper_bound_of_type::<T>();
}

#[inline]
const fn capacity_upper_bound_of_type<T>() -> usize {
  let isize_max_usize = isize::MAX.unsigned_abs();
  if let Some(elem) = isize_max_usize.checked_div(size_of::<T>()) { elem } else { 0 }
}
