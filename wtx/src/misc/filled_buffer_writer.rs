use crate::misc::{Lease, LeaseMut, Vector, VectorError, _unreachable};

/// Helper that manages the copy of initialized bytes.
#[derive(Debug)]
pub struct FilledBufferWriter<'vec> {
  _curr_idx: usize,
  _initial_idx: usize,
  _vec: &'vec mut Vector<u8>,
}

impl<'vec> FilledBufferWriter<'vec> {
  #[inline]
  pub(crate) fn new(start: usize, vec: &'vec mut Vector<u8>) -> Self {
    Self { _curr_idx: start, _initial_idx: start, _vec: vec }
  }

  #[inline]
  pub(crate) fn _curr_bytes(&self) -> &[u8] {
    if let Some(elem) = self._vec.get(self._initial_idx..self._curr_idx) {
      elem
    } else {
      _unreachable()
    }
  }

  #[inline]
  pub(crate) fn _curr_bytes_mut(&mut self) -> &mut [u8] {
    if let Some(elem) = self._vec.get_mut(self._initial_idx..self._curr_idx) {
      elem
    } else {
      _unreachable()
    }
  }

  #[inline]
  pub(crate) fn _len(&self) -> usize {
    self._curr_idx.wrapping_sub(self._initial_idx)
  }

  #[inline]
  pub(crate) fn _extend_from_byte(&mut self, byte: u8) -> Result<(), VectorError> {
    let new_start = self._curr_idx.wrapping_add(1);
    self._expand_buffer(new_start)?;
    let Some(vec_byte) = self._vec.get_mut(self._curr_idx) else {
      _unreachable();
    };
    *vec_byte = byte;
    self._curr_idx = new_start;
    Ok(())
  }

  #[inline]
  pub(crate) fn _extend_from_slice(&mut self, slice: &[u8]) -> Result<(), VectorError> {
    self._extend_from_slice_generic(slice, [])
  }

  #[inline]
  pub(crate) fn _extend_from_slices(&mut self, slices: &[&[u8]]) -> Result<(), VectorError> {
    for slice in slices {
      self._extend_from_slice(slice)?;
    }
    Ok(())
  }

  /// The `c` suffix means that `slice` is copied as a C string.
  #[inline]
  pub(crate) fn _extend_from_slice_c(&mut self, slice: &[u8]) -> Result<(), VectorError> {
    self._extend_from_slice_generic(slice, [0])
  }

  /// The `each_c` suffix means that each slice is copied as a C string.
  #[inline]
  pub(crate) fn _extend_from_slices_each_c(&mut self, slices: &[&[u8]]) -> Result<(), VectorError> {
    for slice in slices {
      self._extend_from_slice_c(slice)?;
    }
    Ok(())
  }

  /// The `rn` suffix means that `slice` is copied with a final `\r\n` new line.
  #[inline]
  pub(crate) fn _extend_from_slice_rn(&mut self, slice: &[u8]) -> Result<(), VectorError> {
    self._extend_from_slice_generic(slice, *b"\r\n")
  }

  /// The `group_rn` suffix means that only the last slice is copied with a final `\r\n` new line.
  #[inline]
  pub(crate) fn _extend_from_slices_group_rn(
    &mut self,
    slices: &[&[u8]],
  ) -> Result<(), VectorError> {
    if let [rest @ .., last] = slices {
      for slice in rest {
        self._extend_from_slice(slice)?;
      }
      self._extend_from_slice_rn(last)?;
    }
    Ok(())
  }

  #[inline]
  pub(crate) fn _shift_idx(&mut self, n: usize) -> Result<(), VectorError> {
    let new_len = self._curr_idx.wrapping_add(n);
    self._expand_buffer(new_len)?;
    self._curr_idx = new_len;
    Ok(())
  }

  #[inline]
  fn _expand_buffer(&mut self, new_len: usize) -> Result<(), VectorError> {
    self._vec.expand(new_len, 0)
  }

  #[inline]
  fn _extend_from_slice_generic<const N: usize>(
    &mut self,
    slice: &[u8],
    suffix: [u8; N],
  ) -> Result<(), VectorError> {
    let until_slice = self._curr_idx.wrapping_add(slice.len());
    let until_suffix = until_slice.wrapping_add(N);
    self._expand_buffer(until_suffix)?;
    if let Some(vec_slice) = self._vec.get_mut(self._curr_idx..until_slice) {
      vec_slice.copy_from_slice(slice);
    } else {
      _unreachable();
    }
    if let Some(vec_slice) = self._vec.get_mut(until_slice..until_suffix) {
      vec_slice.copy_from_slice(&suffix);
    } else {
      _unreachable();
    }
    self._curr_idx = until_suffix;
    Ok(())
  }

  #[inline]
  pub(crate) fn _remaining_bytes_mut(&mut self) -> &mut [u8] {
    if let Some(elem) = self._vec.get_mut(self._curr_idx..) {
      elem
    } else {
      _unreachable()
    }
  }
}

impl<'vec> Lease<FilledBufferWriter<'vec>> for FilledBufferWriter<'vec> {
  #[inline]
  fn lease(&self) -> &FilledBufferWriter<'vec> {
    self
  }
}

impl<'vec> LeaseMut<FilledBufferWriter<'vec>> for FilledBufferWriter<'vec> {
  #[inline]
  fn lease_mut(&mut self) -> &mut FilledBufferWriter<'vec> {
    self
  }
}
