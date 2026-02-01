use crate::{
  collection::{
    LinearStorageLen as _,
    linear_storage::linear_storage_mut::LinearStorageMut,
    misc::{drop_elements, is_char_boundary},
  },
  misc::{Lease, TryArithmetic as _, char_slice},
};
use core::{ptr, slice};

/// Unsized slices like `str` or `[T]`.
pub(crate) trait LinearStorageSlice: Lease<Self> {
  type Data;
  type Unit;

  /// # Safety
  ///
  /// The same safety rules of [`core::slice::from_raw_parts`] apply to this method.
  unsafe fn from_raw_parts<'any>(data: *const Self::Data, len: usize) -> &'any Self;

  /// # Safety
  ///
  /// The same safety rules of [`core::slice::from_raw_parts_mut`] apply to this method.
  unsafe fn from_raw_parts_mut<'any>(data: *mut Self::Data, len: usize) -> &'any mut Self;

  fn data(&self) -> &[Self::Data];

  fn data_from_unit(unit: Self::Unit) -> impl ExactSizeIterator<Item = Self::Data>;

  fn pop<LSM>(lsm: &mut LSM) -> Option<Self::Unit>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self>;

  fn remove<LSM>(lsm: &mut LSM, idx: LSM::Len) -> Option<Self::Unit>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self>;

  fn truncate<LSM>(lsm: &mut LSM, new_len: LSM::Len) -> crate::Result<()>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self> + ?Sized;

  fn units(&self) -> impl DoubleEndedIterator<Item = Self::Unit>
  where
    Self::Unit: Clone;
}

impl LinearStorageSlice for str {
  type Data = u8;
  type Unit = char;

  #[inline]
  unsafe fn from_raw_parts<'any>(data: *const Self::Data, len: usize) -> &'any Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    let slice = unsafe { slice::from_raw_parts(data, len) };
    // SAFETY: it is up to the caller to pass a valid UTF-8 slice
    unsafe { str::from_utf8_unchecked(slice) }
  }

  #[inline]
  unsafe fn from_raw_parts_mut<'any>(data: *mut Self::Data, len: usize) -> &'any mut Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    let slice = unsafe { slice::from_raw_parts_mut(data, len) };
    // SAFETY: it is up to the caller to pass a valid UTF-8 slice
    unsafe { str::from_utf8_unchecked_mut(slice) }
  }

  #[inline]
  fn data(&self) -> &[Self::Data] {
    self.as_bytes()
  }

  #[inline]
  fn data_from_unit(unit: Self::Unit) -> impl ExactSizeIterator<Item = Self::Data> {
    let mut buffer = [0; 4];
    let len = char_slice(&mut buffer, unit).len();
    buffer.into_iter().take(len)
  }

  #[inline]
  fn pop<LSM>(lsm: &mut LSM) -> Option<Self::Unit>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self>,
  {
    let ch = lsm.as_slice().units().next_back()?;
    let new_len = lsm.len().wrapping_sub(LSM::Len::from_usize(ch.len_utf8()).ok()?);
    // SAFETY: Depends on the correct implementation of all associated traits.
    unsafe {
      lsm.set_len(new_len);
    }
    Some(ch)
  }

  #[inline]
  fn remove<LSM>(lsm: &mut LSM, idx: LSM::Len) -> Option<Self::Unit>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self>,
  {
    let rhs = lsm.as_slice().get(idx.usize()..)?;
    let ret = rhs.chars().next()?;
    let next = idx.wrapping_add(LSM::Len::from_usize(ret.len_utf8()).ok()?);
    let len = lsm.len();
    let ptr = lsm.as_ptr_mut();
    // SAFETY: `next` > `idx` and `next` <= `len`
    let src = unsafe { ptr.add(next.usize()) };
    // SAFETY: `idx` < `len`
    let dst = unsafe { ptr.add(idx.usize()) };
    // SAFETY: parameters are valid per above comments
    unsafe {
      ptr::copy(src, dst, len.wrapping_sub(next).usize());
    }
    let new_len = len.wrapping_sub(next.wrapping_sub(idx));
    // SAFETY: `new_len` elements are initialized
    unsafe {
      lsm.set_len(new_len);
    }
    Some(ret)
  }

  #[inline]
  fn truncate<LSM>(lsm: &mut LSM, new_len: LSM::Len) -> crate::Result<()>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self> + ?Sized,
  {
    if new_len > lsm.len() {
      return Ok(());
    }
    if !is_char_boundary(new_len.usize(), lsm.as_slice().as_bytes()) {
      return Err(crate::Error::InvalidUTF8Bound);
    }
    // SAFETY: Depends on the correct implementation of all associated traits.
    unsafe {
      lsm.set_len(new_len);
    }
    Ok(())
  }

  #[inline]
  fn units(&self) -> impl DoubleEndedIterator<Item = Self::Unit>
  where
    Self::Unit: Clone,
  {
    self.chars()
  }
}

impl<T> LinearStorageSlice for [T] {
  type Data = T;
  type Unit = T;

  #[inline]
  unsafe fn from_raw_parts<'any>(data: *const Self::Data, len: usize) -> &'any Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    unsafe { slice::from_raw_parts(data, len) }
  }

  #[inline]
  unsafe fn from_raw_parts_mut<'any>(data: *mut Self::Data, len: usize) -> &'any mut Self {
    // SAFETY: it is up to the caller to pass an initialized slice
    unsafe { slice::from_raw_parts_mut(data, len) }
  }

  #[inline]
  fn data(&self) -> &[Self::Data] {
    self
  }

  #[inline]
  fn data_from_unit(unit: Self::Unit) -> impl ExactSizeIterator<Item = Self::Data> {
    [unit].into_iter()
  }

  #[inline]
  fn pop<LSM>(lsm: &mut LSM) -> Option<Self::Unit>
  where
    LSM: LinearStorageMut<Self::Data>,
  {
    let new_len = lsm.len().try_sub(LSM::Len::ONE).ok()?;
    // SAFETY: collection is not empty
    unsafe {
      lsm.set_len(new_len);
    }
    // SAFETY: collection is not empty
    let src = unsafe { lsm.as_ptr().add(lsm.len().usize()) };
    // SAFETY: collection is not empty
    unsafe { Some(ptr::read(src)) }
  }

  #[inline]
  fn remove<LSM>(lsm: &mut LSM, idx: LSM::Len) -> Option<Self::Unit>
  where
    LSM: LinearStorageMut<Self::Data, Slice = Self>,
  {
    let len = lsm.len();
    if idx >= len {
      return None;
    }
    let ret;
    {
      // SAFETY: `index` is within bounds
      let dst = unsafe { lsm.as_ptr_mut().add(idx.usize()) };
      // SAFETY: `index` is within bounds
      ret = unsafe { ptr::read(dst) };
      let len_minus_one = len.wrapping_sub(LSM::Len::ONE);
      // SAFETY: top-level check ensures a valid pointer
      let src = unsafe { dst.add(1) };
      let count = len_minus_one.wrapping_sub(idx).usize();
      // SAFETY: see safety comments of associated variables
      unsafe {
        ptr::copy(src, dst, count);
      }
      // SAFETY: instance is not empty
      unsafe {
        lsm.set_len(len_minus_one);
      };
    }
    Some(ret)
  }

  #[inline]
  fn truncate<LSM>(lsm: &mut LSM, new_len: LSM::Len) -> crate::Result<()>
  where
    LSM: LinearStorageMut<Self::Data> + ?Sized,
  {
    let len = lsm.len();
    let diff = if let Ok(diff) = len.try_sub(new_len)
      && diff > LSM::Len::ZERO
    {
      diff
    } else {
      return Ok(());
    };
    // SAFETY: indices are within bounds
    unsafe {
      lsm.set_len(new_len);
    }
    if LSM::NEEDS_DROP {
      // SAFETY: indices are within bounds
      unsafe {
        let _rslt = drop_elements(&mut (), diff, new_len, lsm.as_ptr_mut());
      }
    }
    Ok(())
  }

  #[inline]
  fn units(&self) -> impl DoubleEndedIterator<Item = Self::Unit>
  where
    Self::Unit: Clone,
  {
    self.iter().cloned()
  }
}
