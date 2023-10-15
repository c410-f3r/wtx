#![allow(
  // N is not zero
  clippy::arithmetic_side_effects
)]

use core::{
  iter::FusedIterator,
  slice::{self, IterMut},
};

/// Stable in-house version of the `ArrayChunks` struct found in the standard library.
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArrayChunksMut<'slice, T, const N: usize> {
  iter: IterMut<'slice, [T; N]>,
  remainder: &'slice mut [T],
}

impl<'slice, T, const N: usize> ArrayChunksMut<'slice, T, N> {
  #[inline]
  pub(crate) fn new(slice: &'slice mut [T]) -> Self {
    assert!(N != 0, "chunk size must be non-zero");
    let len = slice.len() / N;
    let (multiple_of_n, remainder) = slice.split_at_mut(len * N);
    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    let array_slice = unsafe { slice::from_raw_parts_mut(multiple_of_n.as_mut_ptr().cast(), len) };
    Self { iter: array_slice.iter_mut(), remainder }
  }

  pub(crate) fn into_remainder(self) -> &'slice mut [T] {
    self.remainder
  }
}

impl<'slice, T, const N: usize> DoubleEndedIterator for ArrayChunksMut<'slice, T, N> {
  #[inline]
  fn next_back(&mut self) -> Option<&'slice mut [T; N]> {
    self.iter.next_back()
  }

  #[inline]
  fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
    self.iter.nth_back(n)
  }
}

impl<T, const N: usize> ExactSizeIterator for ArrayChunksMut<'_, T, N> {
  #[inline]
  fn len(&self) -> usize {
    self.iter.len()
  }
}

impl<T, const N: usize> FusedIterator for ArrayChunksMut<'_, T, N> {}

impl<'slice, T, const N: usize> Iterator for ArrayChunksMut<'slice, T, N> {
  type Item = &'slice mut [T; N];

  #[inline]
  fn count(self) -> usize {
    self.iter.count()
  }

  #[inline]
  fn last(self) -> Option<Self::Item> {
    self.iter.last()
  }

  #[inline]
  fn next(&mut self) -> Option<&'slice mut [T; N]> {
    self.iter.next()
  }

  #[inline]
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    self.iter.nth(n)
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
}
