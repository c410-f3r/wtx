use core::{
  iter::FusedIterator,
  mem::size_of,
  slice::{self, Iter, IterMut},
};

#[rustfmt::skip]
macro_rules! create_and_impl {
  (
    $array:ty,
    $from_raw_parts:ident,
    $iter_method:ident,
    $iter_struct:ident,
    $name:ident,
    $ptr_method:ident,
    $split_method:ident,
    $slice:ty
  ) => {
    /// Stable in-house version of the `ArrayChunks` struct found in the standard library.
    #[derive(Debug)]
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    pub struct $name<'slice, T, const N: usize> {
      iter: $iter_struct<'slice, [T; N]>,
      remainder: $slice,
    }

    impl<'slice, T, const N: usize> $name<'slice, T, N> {
      #[allow(
        // N is not zero, therefore, no following arithmetic will panic.
        clippy::arithmetic_side_effects,
      )]
      #[inline]
      /// Returns an iterator over N elements of the slice at a time, starting at the beginning of
      /// the slice.
      pub fn new(slice: $slice) -> Self {
        const {
          if N == 0 || size_of::<T>() == 0 {
            panic!();
          }
        }
        let len = slice.len() / N;
        let (multiple_of_n, remainder) = slice.$split_method(len * N);
        // SAFETY: `N` is not zero and `slice` is multiple of `N`.
        let arrays = unsafe { slice::$from_raw_parts(multiple_of_n.$ptr_method().cast(), len) };
        Self { iter: arrays.$iter_method(), remainder }
      }

      /// Owned version of [Self::remainder] that can return mutable or immutable slices.
      #[inline]
      pub fn into_remainder(self) -> $slice {
        self.remainder
      }

      /// Returns the remainder of the original slice that is not going to be returned by the iterator.
      #[inline]
      pub fn remainder(&self) -> &[T] {
        &self.remainder
      }

      /// Views the underlying data as a subslice of the original data.
      #[inline]
      pub fn slice(&self) -> &[T] {
        let slice = self.iter.as_slice();
        // SAFETY: `T` is not a ZST and the slice is already in the same address space 
        unsafe {
          slice::from_raw_parts(
            slice.as_ptr().cast(),
            slice.len().unchecked_mul(N).unchecked_add(self.remainder.len())
          )
        }
      }
    }

    impl<'slice, T, const N: usize> DoubleEndedIterator for $name<'slice, T, N> {
      #[inline]
      fn next_back(&mut self) -> Option<$array> {
        self.iter.next_back()
      }

      #[inline]
      fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
      }
    }

    impl<T, const N: usize> ExactSizeIterator for $name<'_, T, N> {
      #[inline]
      fn len(&self) -> usize {
        self.iter.len()
      }
    }

    impl<T, const N: usize> FusedIterator for $name<'_, T, N> {}

    impl<'slice, T, const N: usize> Iterator for $name<'slice, T, N> {
      type Item = $array;

      #[inline]
      fn count(self) -> usize {
        self.iter.count()
      }

      #[inline]
      fn last(self) -> Option<Self::Item> {
        self.iter.last()
      }

      #[inline]
      fn next(&mut self) -> Option<$array> {
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
  };
}

create_and_impl!(
  &'slice [T; N],
  from_raw_parts,
  iter,
  Iter,
  ArrayChunks,
  as_ptr,
  split_at,
  &'slice [T]
);
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

#[cfg(test)]
mod tests {
  use crate::misc::ArrayChunks;

  #[test]
  fn basic_usage() {
    let mut iter = ArrayChunks::new(&[1, 2, 3, 4, 5]);
    assert_eq!(iter.slice(), &[1, 2, 3, 4, 5]);
    assert_eq!(iter.remainder(), &[5]);

    assert_eq!(iter.next(), Some(&[1, 2]));
    assert_eq!(iter.slice(), &[3, 4, 5]);
    assert_eq!(iter.remainder(), &[5]);

    assert_eq!(iter.next(), Some(&[3, 4]));
    assert_eq!(iter.slice(), &[5]);
    assert_eq!(iter.remainder(), &[5]);

    assert_eq!(iter.next(), None);
    assert_eq!(iter.slice(), &[5]);
    assert_eq!(iter.remainder(), &[5]);
  }
}
