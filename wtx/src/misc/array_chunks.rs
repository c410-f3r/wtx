#![allow(
  // N is not zero
  clippy::arithmetic_side_effects
)]

use core::{
  iter::FusedIterator,
  slice::{self, IterMut},
};

macro_rules! create_and_impl {
  ($array:ty, $from_raw_parts:ident, $iter_method:ident, $iter_struct:ident, $name:ident, $ptr_method:ident, $split_method:ident, $slice:ty) => {
    /// Stable in-house version of the `ArrayChunks` struct found in the standard library.
    #[derive(Debug)]
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    pub(crate) struct $name<'slice, T, const N: usize> {
      _iter: $iter_struct<'slice, [T; N]>,
      _remainder: $slice,
    }

    impl<'slice, T, const N: usize> $name<'slice, T, N> {
      #[inline]
      pub(crate) fn _new(slice: $slice) -> Self {
        assert!(N != 0, "chunk size must be non-zero");
        let len = slice.len() / N;
        let (multiple_of_n, remainder) = slice.$split_method(len * N);
        #[allow(unsafe_code)]
        // SAFETY: `N` is not zero and `slice` is multiple of `N`.
        let arrays = unsafe { slice::$from_raw_parts(multiple_of_n.$ptr_method().cast(), len) };
        Self { _iter: arrays.$iter_method(), _remainder: remainder }
      }

      pub(crate) fn _into_remainder(self) -> $slice {
        self._remainder
      }
    }

    impl<'slice, T, const N: usize> DoubleEndedIterator for $name<'slice, T, N> {
      #[inline]
      fn next_back(&mut self) -> Option<$array> {
        self._iter.next_back()
      }

      #[inline]
      fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self._iter.nth_back(n)
      }
    }

    impl<T, const N: usize> ExactSizeIterator for $name<'_, T, N> {
      #[inline]
      fn len(&self) -> usize {
        self._iter.len()
      }
    }

    impl<T, const N: usize> FusedIterator for $name<'_, T, N> {}

    impl<'slice, T, const N: usize> Iterator for $name<'slice, T, N> {
      type Item = $array;

      #[inline]
      fn count(self) -> usize {
        self._iter.count()
      }

      #[inline]
      fn last(self) -> Option<Self::Item> {
        self._iter.last()
      }

      #[inline]
      fn next(&mut self) -> Option<$array> {
        self._iter.next()
      }

      #[inline]
      fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self._iter.nth(n)
      }

      #[inline]
      fn size_hint(&self) -> (usize, Option<usize>) {
        self._iter.size_hint()
      }
    }
  };
}

create_and_impl!(
  &'slice mut [T; N],
  from_raw_parts_mut,
  iter_mut,
  IterMut,
  ArrayChunksMut,
  as_mut_ptr,
  split_at_mut,
  &'slice mut [T]
);
