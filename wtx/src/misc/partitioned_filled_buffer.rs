use crate::misc::{filled_buffer::FilledBuffer, FilledBufferWriter, Lease, LeaseMut};
use core::ops::Range;

// ```
// [=========================All=========================]
//
// [=================Buffer=================|            ]
//
// [              |=============Current rest=============]
//
// [                          |======Following rest======]
//
// [==Antecedent==|==Current==|==Following==|==Trailing==]
//                |           |             |            |
//                |           |             |            |--> _buffer.capacity()
//                |           |             |
//                |           |             |---------------> _buffer.len()
//                |           |
//                |           |-----------------------------> _current_end_idx
//                |
//                |-----------------------------------------> _antecedent_end_idx
// ```
#[derive(Debug)]
pub(crate) struct PartitionedFilledBuffer {
  _antecedent_end_idx: usize,
  _buffer: FilledBuffer,
  _current_end_idx: usize,
}

impl PartitionedFilledBuffer {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { _antecedent_end_idx: 0, _buffer: FilledBuffer::_new(), _current_end_idx: 0 }
  }

  #[inline]
  pub(crate) fn _with_capacity(cap: usize) -> crate::Result<Self> {
    Ok(Self {
      _antecedent_end_idx: 0,
      _buffer: FilledBuffer::_with_capacity(cap)?,
      _current_end_idx: 0,
    })
  }

  #[inline]
  pub(crate) fn _antecedent_end_idx(&self) -> usize {
    self._antecedent_end_idx
  }

  #[inline]
  pub(crate) fn _buffer(&self) -> &[u8] {
    self._buffer._all()
  }

  #[inline]
  pub(crate) fn _buffer_mut(&mut self) -> &mut [u8] {
    self._buffer._all_mut()
  }

  #[inline]
  pub(crate) fn _clear(&mut self) {
    let Self { _antecedent_end_idx, _buffer, _current_end_idx } = self;
    *_antecedent_end_idx = 0;
    _buffer._clear();
    *_current_end_idx = 0;
  }

  #[inline]
  pub(crate) fn _clear_if_following_is_empty(&mut self) {
    if !self._has_following() {
      self._clear();
    }
  }

  #[inline]
  pub(crate) fn _current(&self) -> &[u8] {
    let range = self._current_range();
    self._buffer().get(range).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _current_end_idx(&self) -> usize {
    self._current_end_idx
  }

  #[inline]
  pub(crate) fn _current_mut(&mut self) -> &mut [u8] {
    let range = self._current_range();
    self._buffer.get_mut(range).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _current_range(&self) -> Range<usize> {
    self._antecedent_end_idx()..self._current_end_idx()
  }

  #[inline]
  pub(crate) fn _current_rest_mut(&mut self) -> &mut [u8] {
    let idx = self._antecedent_end_idx();
    self._buffer._all_mut().get_mut(idx..).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _following(&self) -> &[u8] {
    let idx = self._current_end_idx();
    self._buffer().get(idx..).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _following_end_idx(&self) -> usize {
    self._buffer.len()
  }

  #[inline]
  pub(crate) fn _following_len(&self) -> usize {
    self._following_end_idx().wrapping_sub(self._current_end_idx())
  }

  #[inline]
  pub(crate) fn _following_mut(&mut self) -> &mut [u8] {
    let idx = self._current_end_idx();
    self._buffer.get_mut(idx..).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _following_rest_mut(&mut self) -> &mut [u8] {
    let idx = self._current_end_idx();
    self._buffer._all_mut().get_mut(idx..).unwrap_or_default()
  }

  #[inline]
  pub(crate) fn _has_following(&self) -> bool {
    self._following_end_idx() > self._current_end_idx()
  }

  #[inline]
  pub(crate) fn _reserve(&mut self, additional: usize) -> crate::Result<()> {
    self._buffer._reserve(additional)
  }

  #[inline]
  pub(crate) fn _set_current(&mut self, bytes: &[u8]) -> crate::Result<()> {
    self._buffer._extend_from_slice(bytes)?;
    self._set_indices(0, bytes.len(), 0)?;
    Ok(())
  }

  #[inline]
  pub(crate) fn _set_indices(
    &mut self,
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> crate::Result<()> {
    let [ant, cur, fol] = Self::_indcs_from_lengths(antecedent_len, current_len, following_len);
    if fol > self._buffer._capacity() {
      return Err(crate::Error::InvalidPartitionedBufferBounds);
    }
    self._antecedent_end_idx = ant;
    self._current_end_idx = cur;
    self._buffer._set_len(fol);
    Ok(())
  }

  #[inline]
  fn _indcs_from_lengths(
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> [usize; 3] {
    let current_end_idx = antecedent_len.saturating_add(current_len);
    let following_end_idx = current_end_idx.saturating_add(following_len);
    [antecedent_len, current_end_idx, following_end_idx]
  }
}

impl Lease<PartitionedFilledBuffer> for PartitionedFilledBuffer {
  #[inline]
  fn lease(&self) -> &PartitionedFilledBuffer {
    self
  }
}

impl LeaseMut<PartitionedFilledBuffer> for PartitionedFilledBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut PartitionedFilledBuffer {
    self
  }
}

impl Default for PartitionedFilledBuffer {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<'pfb> From<&'pfb mut PartitionedFilledBuffer> for FilledBufferWriter<'pfb> {
  #[inline]
  fn from(from: &'pfb mut PartitionedFilledBuffer) -> Self {
    FilledBufferWriter::new(from._following_end_idx(), &mut from._buffer)
  }
}
