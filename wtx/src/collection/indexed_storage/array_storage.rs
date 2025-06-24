use crate::collection::{
  ArrayVectorError, IndexedStorage, StorageLen, indexed_storage::IndexedStorageMut,
};
use core::{mem::MaybeUninit, ptr, slice};

/// Storage backed by an arbitrary array.
#[derive(Debug)]
pub struct ArrayStorage<E, L, const N: usize> {
  len: L,
  elems: [MaybeUninit<E>; N],
}

impl<E, L, const N: usize> ArrayStorage<E, L, N>
where
  L: StorageLen,
{
  /// Constructs a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self::instance_check();
    Self { len: L::ZERO, elems: [const { MaybeUninit::uninit() }; N] }
  }

  const fn instance_check() {
    const {
      assert!(N <= L::CAPACITY_USIZE);
    }
  }

  unsafe fn drop_elements(len: L, offset: L, ptr: *mut E) {
    // SAFETY: It is up to the caller to provide a valid pointer with a valid index
    let data = unsafe { ptr.add(offset.usize()) };
    // SAFETY: It is up to the caller to provide a valid length
    let elements = unsafe { slice::from_raw_parts_mut(data, len.usize()) };
    // SAFETY: It is up to the caller to provide parameters that can lead to droppable elements
    unsafe {
      ptr::drop_in_place(elements);
    }
  }
}

impl<E, L, const N: usize> IndexedStorage<E> for ArrayStorage<E, L, N>
where
  L: StorageLen,
{
  type Len = L;

  #[inline]
  fn as_ptr(&self) -> *const E {
    self.elems.as_ptr().cast()
  }

  #[inline]
  fn len(&self) -> Self::Len {
    self.len
  }
}

impl<E, L, const N: usize> IndexedStorageMut<E> for ArrayStorage<E, L, N>
where
  L: StorageLen,
{
  #[inline]
  fn as_ptr_mut(&mut self) -> *mut E {
    self.elems.as_mut_ptr().cast()
  }

  #[inline]
  fn push(&mut self, elem: E) -> crate::Result<()> {
    let len = self.len;
    if len >= L::CAPACITY {
      return Err(ArrayVectorError::PushOverflow.into());
    }
    // SAFETY: `len` is within `N` bounds
    let dst = unsafe { self.elems.as_mut_ptr().add(len.usize()) };
    // SAFETY: `dst` points to valid uninitialized memory
    unsafe {
      ptr::write(dst, MaybeUninit::new(elem));
    }
    self.len = len.wrapping_add(L::ONE);
    Ok(())
  }

  #[inline]
  fn truncate(&mut self, new_len: Self::Len) {
    let len = self.len;
    let diff = if let Some(diff) = len.checked_sub(new_len)
      && diff > L::ZERO
    {
      diff
    } else {
      return;
    };
    self.len = new_len;
    if Self::NEEDS_DROP {
      // SAFETY: Indices are within bounds
      unsafe {
        Self::drop_elements(diff, new_len, self.as_ptr_mut());
      }
    }
  }
}

impl<E, L, const N: usize> Default for ArrayStorage<E, L, N>
where
  L: StorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
