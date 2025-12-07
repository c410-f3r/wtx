use crate::{
  collection::{
    ExpansionTy, LinearStorageLen as _,
    linear_storage::{LinearStorage, linear_storage_slice::LinearStorageSlice},
  },
  misc::Lease,
};
use core::{ptr, slice};

/// Mutable version of [`LinearStorage`].
pub(crate) trait LinearStorageMut<T>: LinearStorage<T> {
  // ***** REQUIRED *****

  fn as_ptr_mut(&mut self) -> *mut T;

  fn reserve(&mut self, additional: Self::Len) -> crate::Result<()>;

  fn reserve_exact(&mut self, additional: Self::Len) -> crate::Result<()>;

  /// # Safety
  ///
  /// The underlying collection must `new_len` initialized elements.
  unsafe fn set_len(&mut self, new_len: Self::Len);

  // ***** PROVIDED *****

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

  #[inline]
  fn from_cloneable_slice(slice: &Self::Slice) -> crate::Result<Self>
  where
    T: Clone,
    <Self::Slice as LinearStorageSlice>::Unit: Clone,
    Self: Default,
  {
    let mut this = Self::default();
    this.extend_from_cloneable_slice(slice)?;
    Ok(this)
  }

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

  #[inline]
  fn from_iterator(
    iter: impl IntoIterator<Item = <Self::Slice as LinearStorageSlice>::Unit>,
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

  #[inline]
  fn as_slice_mut(&mut self) -> &mut Self::Slice {
    // SAFETY: it is assumed that implementations ensured `self.len()` initialized elements
    unsafe { Self::Slice::from_raw_parts_mut(self.as_ptr_mut(), self.len().usize()) }
  }

  #[inline]
  fn clear(&mut self) {
    let _rslt = Self::Slice::truncate(self, Self::Len::ZERO);
  }

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

  #[inline]
  fn extend_from_cloneable_slice(&mut self, other: &Self::Slice) -> crate::Result<()>
  where
    T: Clone,
    <Self::Slice as LinearStorageSlice>::Unit: Clone,
  {
    for elem in other.units() {
      self.push(elem.clone())?;
    }
    Ok(())
  }

  #[inline]
  fn extend_from_copyable_slice(&mut self, other: &Self::Slice) -> crate::Result<()>
  where
    T: Copy,
  {
    let _ = self.extend_from_copyable_slices([other])?;
    Ok(())
  }

  #[inline]
  fn extend_from_copyable_slices<E, I>(&mut self, others: I) -> crate::Result<Self::Len>
  where
    E: Lease<Self::Slice>,
    I: IntoIterator<Item = E>,
    I::IntoIter: Clone,
    T: Copy,
  {
    let mut others_len_usize: usize = 0;
    let iter = others.into_iter();
    for other in iter.clone() {
      others_len_usize = others_len_usize.wrapping_add(other.lease().data().len());
    }
    let others_len = Self::Len::from_usize(others_len_usize)?;
    self.reserve(others_len)?;
    let mut this_len = self.len();
    for other in iter {
      let other_len_usize = other.lease().data().len();
      // SAFETY: there are allocated elements until `iter_len`
      let dst = unsafe { self.as_ptr_mut().add(this_len.usize()) };
      // SAFETY: both distinct elements have the same length
      unsafe {
        ptr::copy_nonoverlapping(other.lease().data().as_ptr(), dst, other_len_usize);
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

  #[inline]
  fn extend_from_iter(
    &mut self,
    iter: impl IntoIterator<Item = <Self::Slice as LinearStorageSlice>::Unit>,
  ) -> crate::Result<()> {
    for elem in iter {
      self.push(elem)?;
    }
    Ok(())
  }

  #[inline]
  fn push(&mut self, elem: <Self::Slice as LinearStorageSlice>::Unit) -> crate::Result<()> {
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
