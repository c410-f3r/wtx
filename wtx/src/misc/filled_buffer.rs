use crate::{
  collection::{ExpansionTy, Vector},
  misc::{Lease, LeaseMut},
};
use core::{
  fmt::Debug,
  ops::{Deref, DerefMut},
  slice,
};

/// A buffer that is always filled with initialized bytes.
#[derive(Debug, Default)]
pub struct FilledBuffer {
  data: Vector<u8>,
}

impl FilledBuffer {
  #[inline]
  pub(crate) fn _from_vector(mut vector: Vector<u8>) -> Self {
    let prev_init = vector.len();
    // SAFETY: Elements up to `len` are always initialized
    unsafe {
      _fill_remaining_capacity(&mut vector, prev_init);
    }
    Self { data: vector }
  }

  #[inline]
  pub(crate) const fn _new() -> Self {
    Self { data: Vector::new() }
  }

  #[inline]
  pub(crate) fn _with_capacity(cap: usize) -> crate::Result<Self> {
    let mut data = Vector::with_capacity(cap)?;
    // SAFETY: memory have been allocated
    unsafe {
      slice::from_raw_parts_mut(data.as_ptr_mut(), data.capacity()).fill(0);
    }
    Ok(Self { data })
  }

  #[inline]
  pub(crate) fn _all(&self) -> &[u8] {
    // SAFETY: allocated elements are always initialized
    unsafe {
      let len = self.data.capacity();
      slice::from_raw_parts(self.data.as_ptr(), len)
    }
  }

  #[inline]
  pub(crate) fn _all_mut(&mut self) -> &mut [u8] {
    // SAFETY: allocated elements are always initialized
    unsafe {
      let len = self.data.capacity();
      slice::from_raw_parts_mut(self.data.as_ptr_mut(), len)
    }
  }

  #[inline]
  pub(crate) fn _capacity(&self) -> usize {
    self.data.capacity()
  }

  #[inline]
  pub(crate) fn _clear(&mut self) {
    self.data.clear();
  }

  #[inline(always)]
  pub(crate) fn _expand(&mut self, et: ExpansionTy) -> crate::Result<()> {
    let len = self.data.len();
    let Some((additional, new_len)) = et.params(len) else {
      return Ok(());
    };
    self._reserve(additional)?;
    // SAFETY: elements have been initialized
    unsafe {
      self.data.set_len(new_len);
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn _extend_from_slice(&mut self, other: &[u8]) -> crate::Result<()> {
    let _ = self._extend_from_slices([other])?;
    Ok(())
  }

  #[inline]
  pub(crate) fn _extend_from_slices<'iter, I>(&mut self, others: I) -> crate::Result<usize>
  where
    I: IntoIterator<Item = &'iter [u8]>,
    I::IntoIter: Clone,
  {
    let prev_init = self.data.capacity();
    let len = self.data.extend_from_copyable_slices(others)?;
    // SAFETY: Inner elements up to `capacity` are always initialized
    unsafe {
      _fill_remaining_capacity(&mut self.data, prev_init);
    }
    Ok(len)
  }

  #[inline(always)]
  pub(crate) fn _reserve(&mut self, additional: usize) -> crate::Result<()> {
    let prev_init = self.data.capacity();
    self.data.reserve(additional)?;
    // SAFETY: Inner elements up to `capacity` are always initialized
    unsafe {
      _fill_remaining_capacity(&mut self.data, prev_init);
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn _set_len(&mut self, mut len: usize) {
    len = len.min(self.data.capacity());
    // SAFETY: allocated memory is always initialized
    unsafe { self.data.set_len(len) }
  }

  #[inline]
  pub(crate) fn _truncate(&mut self, len: usize) {
    self.data.truncate(len);
  }

  #[inline]
  pub(crate) fn _vector_mut(&mut self) -> FilledBufferVectorMut<'_> {
    FilledBufferVectorMut { prev_init: self.data.capacity(), vector: &mut self.data }
  }
}

impl Deref for FilledBuffer {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.data.as_slice()
  }
}

impl DerefMut for FilledBuffer {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.data.as_slice_mut()
  }
}

impl From<FilledBuffer> for Vector<u8> {
  #[inline]
  fn from(from: FilledBuffer) -> Self {
    from.data
  }
}

impl From<Vector<u8>> for FilledBuffer {
  #[inline]
  fn from(from: Vector<u8>) -> Self {
    Self::_from_vector(from)
  }
}

#[cfg(feature = "std")]
impl std::io::Write for FilledBuffer {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.data.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.data.flush()
  }
}

/// A wrapper that allows the direct usage of [Vector].
#[derive(Debug)]
pub struct FilledBufferVectorMut<'fb> {
  prev_init: usize,
  vector: &'fb mut Vector<u8>,
}

impl Lease<Vector<u8>> for FilledBufferVectorMut<'_> {
  #[inline]
  fn lease(&self) -> &Vector<u8> {
    self.vector
  }
}

impl LeaseMut<Vector<u8>> for FilledBufferVectorMut<'_> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Vector<u8> {
    self.vector
  }
}

impl Drop for FilledBufferVectorMut<'_> {
  #[inline]
  fn drop(&mut self) {
    // SAFETY: Inner elements up to `capacity` are always initialized
    unsafe {
      _fill_remaining_capacity(self.vector, self.prev_init);
    }
  }
}

#[inline]
unsafe fn _fill_remaining_capacity(data: &mut Vector<u8>, prev_init: usize) {
  let count = data.len().max(prev_init);
  let Some(diff @ 1..=usize::MAX) = data.capacity().checked_sub(count) else {
    return;
  };
  // SAFETY: caller must ensure `prev_init` elements
  let ptr = unsafe { data.as_ptr_mut().add(count) };
  // SAFETY: caller must ensure allocated memory
  unsafe {
    slice::from_raw_parts_mut(ptr, diff).fill(0);
  }
}

#[cfg(kani)]
mod kani {
  use crate::misc::filled_buffer::FilledBuffer;

  #[kani::proof]
  fn reserve_is_allocation() {
    let reserve: u8 = kani::any();
    let mut vec = FilledBuffer::_new();
    vec._reserve(reserve.into()).unwrap();
    assert!(vec._capacity() >= reserve.into());
    assert!(vec._all_mut().len() >= reserve.into());
    assert_eq!(vec.len(), 0);
    let len = 16usize.min(reserve.into());
    vec._set_len(len);
    assert_eq!(vec.len(), len);
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::FilledBuffer;

  #[test]
  fn extend_from_slices_with_increasing_cap() {
    let mut vec = FilledBuffer::_new();
    let _ = vec._extend_from_slices([&[1, 2, 3][..]]).unwrap();
    assert_eq!(&*vec, &[1, 2, 3]);
  }
}
