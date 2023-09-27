#![allow(
  // False positive
  clippy::arithmetic_side_effects,
  // Indices are within bounds
  clippy::indexing_slicing
)]

use core::{
  array,
  sync::atomic::{AtomicUsize, Ordering},
};

/// Helper intended to avoid excessive allocations between multiple tasks/threads through
/// the sharing of `N` elements behind some provided locking mechanism.
///
/// Note that the current approach locks the maximum number of simultaneous accesses to `N`. If
/// it is not desirable, you can create your own strategy or always allocate a new instance.
#[derive(Debug)]
pub struct Cache<T, const N: usize> {
  array: [T; N],
  idx: AtomicUsize,
}

impl<T, const N: usize> Cache<T, N> {
  /// It is up to the caller to provide all elements.
  #[inline]
  pub const fn new(array: [T; N]) -> Self {
    Self { array, idx: AtomicUsize::new(0) }
  }

  /// Each array element is constructed using `cb`.
  #[inline]
  pub fn from_cb(cb: impl FnMut(usize) -> T) -> Self {
    Self { array: array::from_fn(cb), idx: AtomicUsize::new(0) }
  }

  /// Provides the next available element returning back to the begging when the internal
  /// counter overflows `N`.
  #[inline]
  pub fn next(&self) -> &T {
    &self.array[self.idx.fetch_add(1, Ordering::Relaxed) & (N - 1)]
  }
}
