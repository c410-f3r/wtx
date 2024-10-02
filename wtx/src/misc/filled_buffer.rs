use crate::misc::{BufferParam, Vector, VectorError};
use core::{
  fmt::Debug,
  ops::{Deref, DerefMut},
  slice,
};

#[derive(Debug, Default)]
pub(crate) struct FilledBuffer {
  data: Vector<u8>,
}

impl FilledBuffer {
  #[inline]
  pub(crate) const fn _new() -> Self {
    Self { data: Vector::new() }
  }

  #[inline]
  pub(crate) fn _with_capacity(cap: usize) -> Result<Self, VectorError> {
    let mut data = Vector::with_capacity(cap)?;
    // SAFETY: memory have been allocated
    unsafe {
      slice::from_raw_parts_mut(data.as_mut_ptr(), data.capacity()).fill(0);
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
      slice::from_raw_parts_mut(self.data.as_mut_ptr(), len)
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
  pub(crate) fn _expand(&mut self, bp: BufferParam) -> Result<(), VectorError> {
    let len = self.data.len();
    let Some((additional, new_len)) = bp.params(len) else {
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
  pub(crate) fn _extend_from_slices<'iter, I>(&mut self, others: I) -> Result<usize, VectorError>
  where
    I: IntoIterator<Item = &'iter [u8]>,
    I::IntoIter: Clone,
  {
    let prev_cap = self.data.capacity();
    let len = self.data.extend_from_slices(others)?;
    // SAFETY: memory have been allocated
    unsafe {
      self._fill_remaining_capacity(prev_cap);
    }
    Ok(len)
  }

  #[inline]
  pub(crate) fn _len(&self) -> usize {
    self.data.len()
  }

  #[inline(always)]
  pub(crate) fn _reserve(&mut self, additional: usize) -> Result<(), VectorError> {
    let prev_cap = self.data.capacity();
    self.data.reserve(additional)?;
    // SAFETY: memory have been allocated
    unsafe {
      self._fill_remaining_capacity(prev_cap);
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
  unsafe fn _fill_remaining_capacity(&mut self, prev_cap: usize) {
    let count = self.data.len().max(prev_cap);
    let Some(diff @ 1..=usize::MAX) = self.data.capacity().checked_sub(count) else {
      return;
    };
    // SAFETY: caller must ensure `prev_cap` elements
    let ptr = unsafe { self.data.as_mut_ptr().add(count) };
    // SAFETY: caller must ensure allocated memory
    unsafe {
      slice::from_raw_parts_mut(ptr, diff).fill(0);
    }
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
    self.data.as_mut_slice()
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

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod proptest {
  use crate::misc::FilledBuffer;

  #[test_strategy::proptest]
  fn reserve_is_allocation(reserve: u8) {
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
