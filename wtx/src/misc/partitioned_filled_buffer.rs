#![allow(
  // Indices point to valid memory
  clippy::unreachable
)]

use core::ops::Range;

use crate::{misc::FilledBufferWriter, DFLT_PARTITIONED_BUFFER_LEN};
use alloc::{vec, vec::Vec};

// ```
// [ Antecedent | Current | Following | Trailing ]
// ```
#[derive(Debug)]
pub(crate) struct PartitionedFilledBuffer {
  _antecedent_end_idx: usize,
  _buffer: Vec<u8>,
  _current_end_idx: usize,
  _following_end_idx: usize,
}

impl PartitionedFilledBuffer {
  pub(crate) fn with_capacity(cap: usize) -> Self {
    Self {
      _antecedent_end_idx: 0,
      _buffer: vec![0; cap],
      _current_end_idx: 0,
      _following_end_idx: 0,
    }
  }

  pub(crate) fn _empty() -> Self {
    Self { _antecedent_end_idx: 0, _buffer: Vec::new(), _current_end_idx: 0, _following_end_idx: 0 }
  }

  pub(crate) fn _antecedent_end_idx(&self) -> usize {
    self._antecedent_end_idx
  }

  pub(crate) fn _buffer(&self) -> &[u8] {
    &self._buffer
  }

  pub(crate) fn _buffer_mut(&mut self) -> &mut [u8] {
    &mut self._buffer
  }

  pub(crate) fn _clear(&mut self) {
    self._antecedent_end_idx = 0;
    self._current_end_idx = 0;
    self._following_end_idx = 0;
  }

  pub(crate) fn _clear_if_following_is_empty(&mut self) {
    if !self._has_following() {
      self._clear();
    }
  }

  /// Current along side any trailing bytes
  pub(crate) fn _current_trail_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self._buffer.get_mut(self._antecedent_end_idx..) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _current(&self) -> &[u8] {
    if let Some(el) = self._buffer.get(self._current_range()) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _current_end_idx(&self) -> usize {
    self._current_end_idx
  }

  pub(crate) fn _current_mut(&mut self) -> &mut [u8] {
    let range = self._current_range();
    if let Some(el) = self._buffer.get_mut(range) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _current_range(&self) -> Range<usize> {
    self._antecedent_end_idx..self._current_end_idx
  }

  pub(crate) fn _expand_buffer(&mut self, new_len: usize) {
    if new_len > self._buffer.len() {
      self._buffer.resize(new_len, 0);
    }
  }

  /// Expands the buffer that can accommodate "following" but doesn't set its length.
  pub(crate) fn _expand_following(&mut self, new_len: usize) {
    self._expand_buffer(self._following_end_idx.wrapping_add(new_len));
  }

  pub(crate) fn _following(&self) -> &[u8] {
    if let Some(el) = self._buffer.get(self._current_end_idx..self._following_end_idx) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _following_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self._buffer.get_mut(self._current_end_idx..self._following_end_idx) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _following_len(&self) -> usize {
    self._following_end_idx.wrapping_sub(self._current_end_idx)
  }

  /// Following bytes along side any trailing bytes
  pub(crate) fn _following_trail_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self._buffer.get_mut(self._current_end_idx..) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _has_following(&self) -> bool {
    self._following_end_idx > self._current_end_idx
  }

  pub(crate) fn _set_indices(
    &mut self,
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> crate::Result<()> {
    let [ant, cur, fol] = Self::_indcs_from_lengths(antecedent_len, current_len, following_len);
    if fol > self._buffer.len() {
      return Err(crate::Error::InvalidPartitionedBufferBounds);
    }
    self._antecedent_end_idx = ant;
    self._current_end_idx = cur;
    self._following_end_idx = fol;
    Ok(())
  }

  pub(crate) fn _set_indices_through_expansion(
    &mut self,
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) {
    let [ant, cur, fol] = Self::_indcs_from_lengths(antecedent_len, current_len, following_len);
    self._antecedent_end_idx = ant;
    self._current_end_idx = cur;
    self._following_end_idx = fol;
    self._expand_buffer(fol);
  }

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

impl Default for PartitionedFilledBuffer {
  #[inline]
  fn default() -> Self {
    Self::with_capacity(DFLT_PARTITIONED_BUFFER_LEN)
  }
}

impl<'pfb> From<&'pfb mut PartitionedFilledBuffer> for FilledBufferWriter<'pfb> {
  #[inline]
  fn from(from: &'pfb mut PartitionedFilledBuffer) -> Self {
    FilledBufferWriter::new(from._following_end_idx, &mut from._buffer)
  }
}
