#![allow(
  // Indices point to valid memory
  clippy::unreachable
)]

use crate::DFLT_PARTITIONED_BUFFER_LEN;
use alloc::{vec, vec::Vec};

/// Internal buffer not intended for public usage
#[derive(Debug)]
pub struct PartitionedBuffer {
  antecedent_end_idx: usize,
  buffer: Vec<u8>,
  current_end_idx: usize,
  following_end_idx: usize,
}

impl PartitionedBuffer {
  pub(crate) fn with_capacity(len: usize) -> Self {
    Self { antecedent_end_idx: 0, buffer: vec![0; len], current_end_idx: 0, following_end_idx: 0 }
  }

  pub(crate) fn _buffer(&self) -> &[u8] {
    &self.buffer
  }

  pub(crate) fn clear(&mut self) {
    self.antecedent_end_idx = 0;
    self.current_end_idx = 0;
    self.following_end_idx = 0;
  }

  pub(crate) fn clear_if_following_is_empty(&mut self) {
    if !self.has_following() {
      self.clear();
    }
  }

  /// Current along side any trailing bytes
  pub(crate) fn current_trail_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self.buffer.get_mut(self.antecedent_end_idx..) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn current(&self) -> &[u8] {
    if let Some(el) = self.buffer.get(self.antecedent_end_idx..self.current_end_idx) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn current_end_idx(&self) -> usize {
    self.current_end_idx
  }

  pub(crate) fn current_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self.buffer.get_mut(self.antecedent_end_idx..self.current_end_idx) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn expand_buffer(&mut self, new_len: usize) {
    if new_len > self.buffer.len() {
      self.buffer.resize(new_len, 0);
    }
  }

  /// Expands the buffer that can accommodate "following" but doesn't set its length.
  pub(crate) fn expand_following(&mut self, new_len: usize) {
    self.expand_buffer(self.following_end_idx.wrapping_add(new_len));
  }

  pub(crate) fn _following(&self) -> &[u8] {
    if let Some(el) = self.buffer.get(self.current_end_idx..self.following_end_idx) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn _following_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self.buffer.get_mut(self.current_end_idx..self.following_end_idx) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn following_len(&self) -> usize {
    self.following_end_idx.wrapping_sub(self.current_end_idx)
  }

  /// Following along side any trailing bytes
  pub(crate) fn following_trail_mut(&mut self) -> &mut [u8] {
    if let Some(el) = self.buffer.get_mut(self.current_end_idx..) {
      el
    } else {
      unreachable!()
    }
  }

  pub(crate) fn has_following(&self) -> bool {
    self.following_end_idx > self.current_end_idx
  }

  pub(crate) fn set_indices(
    &mut self,
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> crate::Result<()> {
    let [ant, cur, fol] = Self::indcs_from_lengths(antecedent_len, current_len, following_len);
    if fol > self.buffer.len() {
      return Err(crate::Error::InvalidPartitionedBufferBounds);
    }
    self.antecedent_end_idx = ant;
    self.current_end_idx = cur;
    self.following_end_idx = fol;
    Ok(())
  }

  pub(crate) fn _set_indices_through_expansion(
    &mut self,
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) {
    let [ant, cur, fol] = Self::indcs_from_lengths(antecedent_len, current_len, following_len);
    self.antecedent_end_idx = ant;
    self.current_end_idx = cur;
    self.following_end_idx = fol;
    self.expand_buffer(fol);
  }

  fn indcs_from_lengths(
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> [usize; 3] {
    let current_end_idx = antecedent_len.saturating_add(current_len);
    let following_end_idx = current_end_idx.saturating_add(following_len);
    [antecedent_len, current_end_idx, following_end_idx]
  }
}

impl Default for PartitionedBuffer {
  #[inline]
  fn default() -> Self {
    Self::with_capacity(DFLT_PARTITIONED_BUFFER_LEN)
  }
}

impl Extend<u8> for PartitionedBuffer {
  #[inline]
  fn extend<T>(&mut self, iter: T)
  where
    T: IntoIterator<Item = u8>,
  {
    self.buffer.extend(iter);
  }
}

impl<'item> Extend<&'item u8> for PartitionedBuffer {
  #[inline]
  fn extend<T>(&mut self, iter: T)
  where
    T: IntoIterator<Item = &'item u8>,
  {
    self.buffer.extend(iter);
  }
}
