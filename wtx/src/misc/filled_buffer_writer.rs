use crate::misc::{filled_buffer::FilledBuffer, BufferMode, Lease, LeaseMut};

/// Helper that manages the copy of initialized bytes.
#[derive(Debug)]
pub struct FilledBufferWriter<'vec> {
  _curr_idx: usize,
  _initial_idx: usize,
  _vec: &'vec mut FilledBuffer,
}

impl<'vec> FilledBufferWriter<'vec> {
  #[inline]
  pub(crate) fn new(start: usize, vec: &'vec mut FilledBuffer) -> Self {
    Self { _curr_idx: start, _initial_idx: start, _vec: vec }
  }

  /// Iterates over the slice `other`, copies each element, and then appends
  /// it to this vector. The `other` slice is traversed in-order.
  #[inline]
  pub fn extend_from_slice(&mut self, other: &[u8]) -> crate::Result<()> {
    self._extend_from_slices([other])
  }

  #[inline]
  pub(crate) fn _curr_bytes(&self) -> &[u8] {
    self._vec.get(self._initial_idx..self._curr_idx).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _curr_bytes_mut(&mut self) -> &mut [u8] {
    self._vec.get_mut(self._initial_idx..self._curr_idx).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _len(&self) -> usize {
    self._curr_idx.wrapping_sub(self._initial_idx)
  }

  #[inline]
  pub(crate) fn _extend_from_byte(&mut self, byte: u8) -> crate::Result<()> {
    self._extend_from_slices([&[byte][..]])
  }

  #[inline]
  pub(crate) fn _extend_from_slices<'iter, I>(&mut self, slices: I) -> crate::Result<()>
  where
    I: IntoIterator<Item = &'iter [u8]>,
    I::IntoIter: Clone,
  {
    let sum = self._vec._extend_from_slices(slices)?;
    self._curr_idx = self._curr_idx.wrapping_add(sum);
    Ok(())
  }

  /// The `c` suffix means that `slice` is copied as a C string.
  #[inline]
  pub(crate) fn _extend_from_slice_c(&mut self, slice: &[u8]) -> crate::Result<()> {
    self._extend_from_slices([slice, &[0]])
  }

  /// The `each_c` suffix means that each slice is copied as a C string.
  #[inline]
  pub(crate) fn _extend_from_slices_each_c(&mut self, slices: &[&[u8]]) -> crate::Result<()> {
    self._extend_from_slices(slices.iter().flat_map(|el| [*el, &[0]]))
  }

  /// The `rn` suffix means that `slice` is copied with a final `\r\n` new line.
  #[inline]
  pub(crate) fn _extend_from_slice_rn(&mut self, slice: &[u8]) -> crate::Result<()> {
    self._extend_from_slices([slice, "\r\n".as_bytes()])
  }

  /// The `group_rn` suffix means that only the last slice is copied with a final `\r\n` new line.
  #[inline]
  pub(crate) fn _extend_from_slices_group_rn(&mut self, slices: &[&[u8]]) -> crate::Result<()> {
    self._extend_from_slices(slices.iter().copied().chain(["\r\n".as_bytes()]))
  }

  #[inline]
  pub(crate) fn _remaining_bytes_mut(&mut self) -> &mut [u8] {
    self._vec._all_mut().get_mut(self._curr_idx..).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _shift_idx(&mut self, n: usize) -> crate::Result<()> {
    let new_len = self._curr_idx.wrapping_add(n);
    self._vec._expand(BufferMode::Len(new_len))?;
    self._curr_idx = new_len;
    Ok(())
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

impl Drop for FilledBufferWriter<'_> {
  #[inline]
  fn drop(&mut self) {
    self._vec._truncate(self._initial_idx);
  }
}

#[cfg(feature = "std")]
impl std::io::Write for FilledBufferWriter<'_> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.extend_from_slice(buf).map_err(std::io::Error::other)?;
    Ok(buf.len())
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self._vec.flush()
  }
}
