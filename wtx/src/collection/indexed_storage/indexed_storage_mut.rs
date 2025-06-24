use crate::collection::{IndexedStorage, indexed_storage::storage_len::StorageLen as _};
use alloc::vec::Vec;
use core::slice;

/// Mutable version of [`IndexedStorage`].
pub trait IndexedStorageMut<E>: IndexedStorage<E> {
  /// Mutable version of [`IndexedStorage::as_ptr`].
  fn as_ptr_mut(&mut self) -> *mut E;

  /// Mutable version of [`IndexedStorage::as_slice`].
  #[inline]
  fn as_slice_mut(&mut self) -> &mut [E] {
    // SAFETY: It is assumed that implementations ensured `self.len()` initialized elements
    unsafe { slice::from_raw_parts_mut(self.as_ptr_mut(), self.len().usize()) }
  }

  /// Clears the storage, removing all values.
  #[inline]
  fn clear(&mut self) {
    self.truncate(Self::Len::ZERO);
  }

  /// Appends an element to the back of the collection.
  fn push(&mut self, elem: E) -> crate::Result<()>;

  /// Shortens the storage, keeping the first `new_len` elements.
  fn truncate(&mut self, new_len: Self::Len);
}

impl<E, T> IndexedStorageMut<E> for &mut T
where
  T: IndexedStorageMut<E>,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut E {
    (**self).as_ptr_mut()
  }

  #[inline]
  fn push(&mut self, elem: E) -> crate::Result<()> {
    (**self).push(elem)
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    (**self).truncate(new_len);
  }
}

impl<E> IndexedStorageMut<E> for Vec<E> {
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut E {
    self.as_mut_ptr()
  }

  #[inline]
  fn push(&mut self, elem: E) -> crate::Result<()> {
    self.push(elem);
    Ok(())
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    self.truncate(new_len);
  }
}
