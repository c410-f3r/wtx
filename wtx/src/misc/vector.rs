macro_rules! extend_from_slices {
  ($iter:expr, $len:expr, $this:expr) => {
    if $len > 8 {
      _unreachable();
    }
    let mut len: usize = 0;
    for other in $iter {
      #[cfg(feature = "nightly")]
      unsafe {
        len = len.unchecked_add(other.as_ref().len());
      }
      #[cfg(not(feature = "nightly"))]
      {
        len = len.wrapping_add(other.as_ref().len());
      }
    }
    $this.reserve(len);
    for other in $iter {
      $this.extend_from_slice_within_cap(other.as_ref());
    }
  };
}

use crate::misc::_unreachable;
use alloc::vec::Vec;
use core::{
  ops::{Deref, DerefMut},
  ptr,
};

/// A wrapper around the std's vector with some additional methods to manipulate copyable data.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Vector<D> {
  data: Vec<D>,
}

impl<D> Vector<D>
where
  D: Copy,
{
  /// Constructs a new, empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Vec::new() }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(cap: usize) -> Self {
    let data = Vec::with_capacity(cap);
    #[cfg(feature = "nightly")]
    unsafe {
      core::hint::assert_unchecked(data.len().unchecked_add(cap) <= data.capacity());
    }
    Self { data }
  }

  /// Returns an unsafe mutable pointer to the vector's buffer, or a dangling
  /// raw pointer valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_mut_ptr(&mut self) -> *mut D {
    self.data.as_mut_ptr()
  }

  /// Returns a raw pointer to the vector's buffer, or a dangling raw pointer
  /// valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_ptr(&self) -> *const D {
    self.data.as_ptr()
  }

  /// Returns the total number of elements the vector can hold without reallocating.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.data.capacity()
  }

  /// Clears the vector, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    self.data.clear()
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  ///
  /// # Panics
  ///
  /// If memory reservation fails.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[D]) {
    self.reserve(other.len());
    self.extend_from_slice_within_cap(other)
  }

  /// Generalization of [Self::extend_from_slice_within_cap].
  ///
  /// # Panics
  ///
  /// If memory reservation fails or if the length of `others` is greater than 8.
  pub fn extend_from_slices<U>(&mut self, others: &[U])
  where
    U: AsRef<[D]>,
  {
    extend_from_slices!(others, others.len(), self);
  }

  /// Generalization of [Self::extend_from_slice_within_cap] for slices with optional values.
  ///
  /// # Panics
  ///
  /// If memory reservation fails or if the length of `others` is greater than 8.
  pub fn extend_from_slices_opt<U>(&mut self, others: &[Option<U>])
  where
    U: AsRef<[D]>,
  {
    extend_from_slices!(others.iter().flatten(), others.len(), self);
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  ///
  /// # Panics
  ///
  /// If there is no available capacity.
  #[inline]
  pub fn extend_from_slice_within_cap(&mut self, other: &[D]) {
    let len = self.data.len();
    let other_len = other.len();
    unsafe {
      #[cfg(feature = "nightly")]
      let new_len = len.unchecked_add(other_len);
      #[cfg(not(feature = "nightly"))]
      let new_len = len.wrapping_add(other_len);

      if new_len > self.data.capacity() {
        panic!("Must be called with sufficient capacity");
      }
      ptr::copy_nonoverlapping(other.as_ptr(), self.data.as_mut_ptr().add(len), other_len);
      self.data.set_len(new_len);
    }
  }

  /// Appends an element to the back of the collection.
  ///
  /// # Panics
  ///
  /// If memory reservation fails.
  #[inline]
  pub fn push(&mut self, value: D) {
    self.reserve(1);
    self.push_within_cap(value);
  }

  /// Appends an element to the back of the collection.
  ///
  /// # Panics
  ///
  /// If there is no available capacity.
  #[inline]
  pub fn push_within_cap(&mut self, value: D) {
    if self.data.len() >= self.data.capacity() {
      _unreachable();
    }
    unsafe {
      ptr::write(self.data.as_mut_ptr().add(self.data.len()), value);

      #[cfg(feature = "nightly")]
      let new_len = self.data.len().unchecked_add(1);
      #[cfg(not(feature = "nightly"))]
      let new_len = self.data.len().wrapping_add(1);

      self.data.set_len(new_len);
    }
  }

  /// Reserves capacity for at least `additional` more elements to be inserted
  /// in the given instance. The collection may reserve more space to
  /// speculatively avoid frequent reallocations. After calling `reserve`,
  /// capacity will be greater than or equal to `self.len() + additional`.
  /// Does nothing if capacity is already sufficient.
  ///
  /// # Panics
  ///
  /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    self.data.reserve(additional);
    #[cfg(feature = "nightly")]
    unsafe {
      core::hint::assert_unchecked(
        self.data.len().unchecked_add(additional) <= self.data.capacity(),
      );
    }
  }

  /// Forces the length of the vector to `new_len`.
  ///
  /// # Safety
  ///
  /// - `new_len` must be less than or equal to the capacity.
  /// - The elements at `prev_len..new_len` must be initialized.
  #[inline]
  pub(crate) unsafe fn set_len(&mut self, new_len: usize) {
    self.data.set_len(new_len)
  }
}

impl<D> AsRef<[D]> for Vector<D> {
  #[inline]
  fn as_ref(&self) -> &[D] {
    self.data.as_slice()
  }
}

impl<D> Deref for Vector<D>
where
  D: Copy,
{
  type Target = [D];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.data.as_slice()
  }
}

impl<D> DerefMut for Vector<D>
where
  D: Copy,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.as_mut_slice()
  }
}

#[cfg(feature = "_bench")]
#[cfg(test)]
mod bench {
  use crate::misc::vector::Vector;
  use alloc::vec::Vec;

  macro_rules! extend_from_slice {
    (
      $instance:expr,
      $extend_from_slice_method:ident,
      $reserve_method:ident
    ) => {
      $instance.$reserve_method(16 * 8);
      extend_from_slice!(
        @$instance,
        $extend_from_slice_method,
        $reserve_method,
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
      );
    };
    (
      @$instance:expr,
      $extend_from_slice_method:ident,
      $reserve_method:ident,
      $($n:literal),*
    ) => {
      $(
        let _ = $n;
        $instance.$extend_from_slice_method(&[0, 1, 2, 4, 5, 6, 7]);
      )*
    };
  }

  macro_rules! push {
    (
      $instance:expr,
      $push_method:ident,
      $reserve_method:ident
    ) => {
      $instance.$reserve_method(64);
      push!(
        @$instance,
        $push_method,
        $reserve_method,
        01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60,
        61, 62, 63, 64
      )
    };
    (
      @$instance:expr,
      $push_method:ident,
      $reserve_method:ident,
      $($n:literal),*
    ) => {
      $($instance.$push_method($n);)*
    };
  }

  #[bench]
  fn extend_from_slice(b: &mut test::Bencher) {
    let mut vec = Vec::default();
    b.iter(|| {
      extend_from_slice!(vec, extend_from_slice, reserve);
    });
  }

  #[bench]
  fn extend_from_slice_within_cap(b: &mut test::Bencher) {
    let mut vec = Vector::default();
    b.iter(|| {
      extend_from_slice!(vec, extend_from_slice_within_cap, reserve);
    });
  }

  #[bench]
  fn push(b: &mut test::Bencher) {
    let mut vec = Vec::default();
    b.iter(|| {
      push!(vec, push, reserve);
    });
  }

  #[bench]
  fn push_within_cap(b: &mut test::Bencher) {
    let mut vec = Vector::default();
    b.iter(|| {
      push!(vec, push_within_cap, reserve);
    });
  }
}
