use crate::collection::{ExpansionTy, IndexedStorage, IndexedStorageLen as _, IndexedStorageSlice};
use alloc::vec::Vec;
use core::{ptr, slice};

/// Mutable version of [`IndexedStorage`].
pub trait IndexedStorageMut<T>: IndexedStorage<T> {
  // ***** REQUIRED *****

  /// Mutable version of [`IndexedStorage::as_ptr`].
  fn as_ptr_mut(&mut self) -> *mut T;

  /// Shortens the instance, removing the last element.
  fn pop(&mut self) -> Option<<Self::Slice as IndexedStorageSlice>::Unit>;

  /// Reserves capacity for at least `additional` more elements to be inserted
  /// in the given instance. The collection may reserve more space to
  /// speculatively avoid frequent reallocations. After calling `reserve`,
  /// capacity will be greater than or equal to `self.len() + additional`.
  /// Does nothing if capacity is already sufficient.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::<u8>::new();
  /// vec.reserve(10);
  /// assert!(vec.capacity() >= 10);
  /// ```
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()>;

  /// Forces the length of the instance to `new_len`.
  ///
  /// # Safety
  ///
  /// The underlying collection must `new_len` initialized elements.
  unsafe fn set_len(&mut self, new_len: Self::Len);

  /// Shortens the instance, keeping the first len elements and dropping the rest.
  ///
  /// ```
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::from_iter(1u8..6).unwrap();
  /// vec.truncate(2);
  /// assert_eq!(vec.as_slice(), [1, 2]);
  /// ```
  fn truncate(&mut self, new_len: Self::Len);

  // ***** PROVIDED *****

  /// Constructs a new instance with elements provided by `iter`.
  #[inline]
  fn from_cloneable_elem(len: usize, value: T) -> crate::Result<Self>
  where
    T: Clone,
    Self: Default,
  {
    let mut this = Self::default();
    this.expand(ExpansionTy::Len(len), value)?;
    Ok(this)
  }

  /// Creates a new instance from the cloneable elements of `slice`.
  #[inline]
  fn from_cloneable_slice(slice: &Self::Slice) -> crate::Result<Self>
  where
    T: Clone,
    <Self::Slice as IndexedStorageSlice>::Unit: Clone,
    Self: Default,
  {
    let mut this = Self::default();
    this.extend_from_cloneable_slice(slice)?;
    Ok(this)
  }

  /// Creates a new instance from the copyable elements of `slice`.
  #[inline]
  fn from_copyable_slice(slice: &Self::Slice) -> crate::Result<Self>
  where
    T: Copy,
    Self: Default,
  {
    let mut this = Self::default();
    this.extend_from_copyable_slice(slice)?;
    Ok(this)
  }

  /// Constructs a new instance with elements provided by `iter`.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::from_iter(0u8..2).unwrap();
  /// assert_eq!(vec.as_slice(), &[0, 1]);
  /// ```
  #[inline]
  fn from_iter(
    iter: impl IntoIterator<Item = <Self::Slice as IndexedStorageSlice>::Unit>,
  ) -> crate::Result<Self>
  where
    Self: Default,
  {
    let mut this = Self::default();
    for elem in iter {
      this.push(elem)?;
    }
    Ok(this)
  }

  /// Mutable version of [`IndexedStorage::as_slice`].
  ///
  /// ```rust
  /// use wtx::collection::IndexedStorageMut;
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.push(1u8).unwrap();
  /// assert_eq!(vec.as_slice_mut(), &mut [1]);
  /// ```
  #[inline]
  fn as_slice_mut(&mut self) -> &mut Self::Slice {
    // SAFETY: it is assumed that implementations ensured `self.len()` initialized elements
    unsafe { Self::Slice::from_raw_parts_mut(self.as_ptr_mut(), self.len().usize()) }
  }

  /// Clears the instance, removing all values.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.push(1u8);
  /// assert_eq!(vec.len(), 1);
  /// vec.clear();
  /// assert_eq!(vec.len(), 0);
  /// ```
  #[inline]
  fn clear(&mut self) {
    self.truncate(Self::Len::ZERO);
  }

  /// Resizes the instance in-place so that the current length is equal to `et`.
  ///
  /// Does nothing if the calculated length is equal or less than the current length.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.expand(wtx::collection::ExpansionTy::Len(4), 0u8).unwrap();
  /// assert_eq!(vec.len(), 4);
  /// ```
  #[inline]
  fn expand(&mut self, et: ExpansionTy, value: T) -> crate::Result<()>
  where
    T: Clone,
  {
    let len = self.len();
    let Some((additional_usize, new_len_usize)) = et.params(len.usize()) else {
      return Ok(());
    };
    let additional = Self::Len::from_usize(additional_usize)?;
    let new_len = Self::Len::from_usize(new_len_usize)?;
    self.reserve(additional)?;
    // SAFETY: there are initialized elements until `len`
    let ptr = unsafe { self.as_ptr_mut().add(len.usize()) };
    // SAFETY: memory has been allocated
    unsafe {
      slice::from_raw_parts_mut(ptr, additional_usize).fill(value);
    }
    // SAFETY: elements have been initialized
    unsafe {
      self.set_len(new_len);
    }
    Ok(())
  }

  /// Iterates over the slice `other`, clones each element and then appends
  /// it to this instance. The `other` slice is traversed in-order.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.extend_from_cloneable_slice(&[0][..]);
  /// assert_eq!(vec.as_slice(), &[0]);
  /// ```
  #[inline]
  fn extend_from_cloneable_slice(&mut self, other: &Self::Slice) -> crate::Result<()>
  where
    T: Clone,
    <Self::Slice as IndexedStorageSlice>::Unit: Clone,
  {
    for elem in other.units() {
      self.push(elem.clone())?;
    }
    Ok(())
  }

  /// Iterates over the slice `other`, copies each element and then appends
  /// it to this instance. The `other` slice is traversed in-order.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.extend_from_copyable_slice(&[0][..]);
  /// assert_eq!(vec.as_slice(), &[0]);
  /// ```
  #[inline]
  fn extend_from_copyable_slice(&mut self, other: &Self::Slice) -> crate::Result<()>
  where
    T: Copy,
  {
    let _ = self.extend_from_copyable_slices([other])?;
    Ok(())
  }

  /// Generalization of [`IndexedStorageMut::extend_from_copyable_slice`].
  ///
  /// Returns the sum of the lengths of all slices.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.extend_from_copyable_slices([&[0][..], &[1][..]]);
  /// assert_eq!(vec.as_slice(), &[0, 1]);
  /// ```
  #[inline]
  fn extend_from_copyable_slices<'iter, I>(&mut self, others: I) -> crate::Result<Self::Len>
  where
    I: IntoIterator<Item = &'iter Self::Slice>,
    I::IntoIter: Clone,
    T: Copy,
    <Self as IndexedStorage<T>>::Slice: 'iter,
  {
    let mut others_len_usize: usize = 0;
    let iter = others.into_iter();
    for other in iter.clone() {
      others_len_usize = others_len_usize.wrapping_add(other.data().len());
    }
    let others_len = Self::Len::from_usize(others_len_usize)?;
    self.reserve(others_len)?;
    let mut this_len = self.len();
    for other in iter {
      let other_len_usize = other.data().len();
      // SAFETY: there are allocated elements until `iter_len`
      let dst = unsafe { self.as_ptr_mut().add(this_len.usize()) };
      // SAFETY: both distinct elements have the same length
      unsafe {
        ptr::copy_nonoverlapping(other.data().as_ptr(), dst, other_len_usize);
      }
      // The initial check makes this conversion infallible
      this_len = this_len.wrapping_add(Self::Len::from_usize(other_len_usize).unwrap_or_default());
    }
    // SAFETY: everything until `total_len` has been copied from `others`
    unsafe {
      self.set_len(this_len);
    }
    Ok(others_len)
  }

  /// Appends all elements of the iterator.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.extend_from_iter(0..2);
  /// assert_eq!(vec.as_slice(), &[0, 1]);
  /// ```
  #[inline]
  fn extend_from_iter(
    &mut self,
    iter: impl IntoIterator<Item = <Self::Slice as IndexedStorageSlice>::Unit>,
  ) -> crate::Result<()> {
    for elem in iter {
      self.push(elem)?;
    }
    Ok(())
  }

  /// Appends an element to the back of the collection.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.push(3);
  /// assert_eq!(vec.as_slice(), [3]);
  /// ```
  #[inline]
  fn push(&mut self, elem: <Self::Slice as IndexedStorageSlice>::Unit) -> crate::Result<()> {
    let data = Self::Slice::data_from_unit(elem);
    let elem_len = Self::Len::from_usize(data.len())?;
    self.reserve(elem_len)?;
    let this_len = self.len();
    for (idx, local_data) in data.enumerate() {
      let count = this_len.usize().wrapping_add(idx);
      // SAFETY: memory has been allocated and it is up to the implementation to provide a
      //         valid iterator length
      let dst = unsafe { self.as_ptr_mut().add(count) };
      // SAFETY: it is up to the implementation to provide a valid iterator length
      unsafe {
        ptr::write(dst, local_data);
      }
    }
    let new_len = this_len.wrapping_add(elem_len);
    // SAFETY: memory has been allocated and filled up until `new_len`
    unsafe {
      self.set_len(new_len);
    }
    Ok(())
  }
}

impl<T, U> IndexedStorageMut<T> for &mut U
where
  U: IndexedStorageMut<T>,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    (**self).as_ptr_mut()
  }

  #[inline]
  fn pop(&mut self) -> Option<<Self::Slice as IndexedStorageSlice>::Unit> {
    (**self).pop()
  }

  #[inline]
  fn push(&mut self, elem: <Self::Slice as IndexedStorageSlice>::Unit) -> crate::Result<()> {
    (**self).push(elem)
  }

  #[inline]
  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()> {
    (**self).reserve(additional)
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    // SAFETY: delegated to `U`
    unsafe { (**self).set_len(new_len) }
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    (**self).truncate(new_len);
  }
}

impl<T> IndexedStorageMut<T> for Vec<T> {
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut T {
    self.as_mut_ptr()
  }

  #[inline]
  fn pop(&mut self) -> Option<T> {
    self.pop()
  }

  #[inline]
  fn push(&mut self, elem: T) -> crate::Result<()> {
    self.push(elem);
    Ok(())
  }

  #[inline]
  fn reserve(&mut self, additional: usize) -> crate::Result<()> {
    self.reserve(additional);
    Ok(())
  }

  #[inline]
  unsafe fn set_len(&mut self, new_len: Self::Len) {
    // SAFETY: delegated to underlying collection
    unsafe { self.set_len(new_len) }
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    self.truncate(new_len);
  }
}
