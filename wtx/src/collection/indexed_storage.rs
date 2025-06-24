mod array_storage;
mod indexed_storage_mut;
mod storage_len;

use alloc::vec::Vec;
pub use array_storage::*;
use core::{mem::needs_drop, slice};
pub use indexed_storage_mut::*;
pub use storage_len::*;

/// A storage that can be represented through a contiguous segment of memory.
pub trait IndexedStorage<E> {
  /// See [`needs_drop`].
  const NEEDS_DROP: bool = needs_drop::<E>();

  /// See [`StorageLen`].
  type Len: StorageLen;

  /// Returns a raw pointer to the slice’s buffer.
  fn as_ptr(&self) -> *const E;

  /// Returns a slice containing all initialized elements.
  #[inline]
  fn as_slice(&self) -> &[E] {
    // SAFETY: It is assumed that implementations ensured `self.len()` initialized elements
    unsafe { slice::from_raw_parts(self.as_ptr(), self.len().usize()) }
  }

  /// Returns the number of elements in the storage, also referred to as its ‘length’.
  fn len(&self) -> Self::Len;
}

impl<E, T> IndexedStorage<E> for &T
where
  T: IndexedStorage<E>,
{
  type Len = T::Len;

  #[inline]
  fn as_ptr(&self) -> *const E {
    (*self).as_ptr()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    (*self).len()
  }
}

impl<E, T> IndexedStorage<E> for &mut T
where
  T: IndexedStorage<E>,
{
  type Len = T::Len;

  #[inline]
  fn as_ptr(&self) -> *const E {
    (**self).as_ptr()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    (**self).len()
  }
}

impl<E> IndexedStorage<E> for Vec<E> {
  type Len = usize;

  #[inline]
  fn as_ptr(&self) -> *const E {
    self.as_ptr()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len()
  }
}
