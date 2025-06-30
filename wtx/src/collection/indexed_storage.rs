pub(crate) mod indexed_storage_len;
pub(crate) mod indexed_storage_mut;
pub(crate) mod indexed_storage_slice;

use alloc::vec::Vec;
use core::mem::needs_drop;

/// A storage that can be represented through a contiguous segment of memory.
pub trait IndexedStorage<T> {
  /// See [`needs_drop`].
  const NEEDS_DROP: bool = needs_drop::<T>();

  /// See [`indexed_storage_len::IndexedStorageLen`].
  type Len: indexed_storage_len::IndexedStorageLen;
  /// See [`indexed_storage_slice::IndexedStorageSlice`].
  type Slice: indexed_storage_slice::IndexedStorageSlice<Data = T> + ?Sized;

  // ***** REQUIRED *****

  /// Returns a raw pointer to the slice’s buffer.
  fn as_ptr(&self) -> *const T;

  /// Returns the number of elements in the storage, also referred to as its ‘length’.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// assert_eq!(vec.len(), 0);
  /// vec.push(1u8);
  /// assert_eq!(vec.len(), 1);
  /// ```
  fn len(&self) -> Self::Len;

  // ***** PROVIDED *****

  /// Extracts a slice containing the entire instance.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// vec.push(1u8).unwrap();
  /// assert_eq!(vec.as_slice(), &[1]);
  /// ```
  #[inline]
  fn as_slice(&self) -> &Self::Slice {
    use indexed_storage_len::IndexedStorageLen as _;
    use indexed_storage_slice::IndexedStorageSlice as _;
    // SAFETY: it is assumed that implementations ensured `self.len()` initialized elements
    unsafe { Self::Slice::from_raw_parts(self.as_ptr(), self.len().usize()) }
  }

  /// Returns the total number of elements the instance can hold without reallocating.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::Vector::new();
  /// assert_eq!(vec.capacity(), 0);
  /// vec.push(1u8);
  /// assert!(vec.capacity() >= 1);
  /// ```
  fn capacity(&self) -> Self::Len;

  /// How many elements can be added to this collection.
  ///
  /// ```rust
  /// use wtx::collection::{IndexedStorage, IndexedStorageMut};
  /// let mut vec = wtx::collection::ArrayVectorU8::<f32, 4>::new();
  /// assert_eq!(vec.remaining(), 4);
  /// vec.push(1.0);
  /// assert_eq!(vec.remaining(), 3);
  /// ```
  #[inline]
  fn remaining(&self) -> Self::Len {
    use indexed_storage_len::IndexedStorageLen as _;
    self.capacity().wrapping_sub(self.len())
  }
}

impl<T, U> IndexedStorage<T> for &U
where
  U: IndexedStorage<T>,
{
  type Len = U::Len;
  type Slice = U::Slice;

  #[inline]
  fn as_ptr(&self) -> *const T {
    (*self).as_ptr()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    (*self).capacity()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    (*self).len()
  }
}

impl<T, U> IndexedStorage<T> for &mut U
where
  U: IndexedStorage<T>,
{
  type Len = U::Len;
  type Slice = U::Slice;

  #[inline]
  fn as_ptr(&self) -> *const T {
    (**self).as_ptr()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    (**self).capacity()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    (**self).len()
  }
}

impl<T> IndexedStorage<T> for Vec<T> {
  type Len = usize;
  type Slice = [T];

  #[inline]
  fn as_ptr(&self) -> *const T {
    self.as_ptr()
  }

  #[inline]
  fn capacity(&self) -> Self::Len {
    self.capacity()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len()
  }
}
