pub(crate) mod linear_storage_len;
pub(crate) mod linear_storage_mut;
pub(crate) mod linear_storage_slice;

use core::mem::needs_drop;

/// A storage that can be represented through a contiguous segment of memory.
pub(crate) trait LinearStorage<T> {
  /// See [`needs_drop`].
  const NEEDS_DROP: bool = needs_drop::<T>();

  /// See [`indexed_storage_len::LinearStorageLen`].
  type Len: linear_storage_len::LinearStorageLen;
  /// See [`indexed_storage_slice::LinearStorageSlice`].
  type Slice: linear_storage_slice::LinearStorageSlice<Data = T> + ?Sized;

  // ***** REQUIRED *****

  fn as_ptr(&self) -> *const T;

  fn capacity(&self) -> Self::Len;

  fn len(&self) -> Self::Len;

  // ***** PROVIDED *****

  #[inline]
  fn as_slice(&self) -> &Self::Slice {
    use linear_storage_len::LinearStorageLen as _;
    use linear_storage_slice::LinearStorageSlice as _;
    // SAFETY: it is assumed that implementations ensured `self.len()` initialized elements
    unsafe { Self::Slice::from_raw_parts(self.as_ptr(), self.len().usize()) }
  }

  #[inline]
  fn remaining(&self) -> Self::Len {
    use linear_storage_len::LinearStorageLen as _;
    self.capacity().wrapping_sub(self.len())
  }
}

impl<T> LinearStorage<T> for alloc::vec::Vec<T> {
  type Len = usize;
  type Slice = [T];

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
