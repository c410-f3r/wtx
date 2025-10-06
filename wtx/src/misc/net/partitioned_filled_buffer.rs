use crate::misc::{FilledBuffer, Lease, LeaseMut};
use core::ops::Range;

// ```
// [=================All=================]
//
// [===========Buffer===========|        ]
//
// [          |=======Current rest=======]
//
// [                  |==Following rest==]
//
// [Antecedent|Current|Following|Trailing]
//            |       |         |        |
//            |       |         |        |-> buffer.capacity()
//            |       |         |
//            |       |         |----------> following_end_idx (buffer.len())
//            |       |
//            |       |--------------------> current_end_idx
//            |
//            |----------------------------> antecedent_end_idx
// ```
#[derive(Debug)]
pub(crate) struct PartitionedFilledBuffer {
  antecedent_end_idx: usize,
  buffer: FilledBuffer,
  current_end_idx: usize,
}

impl PartitionedFilledBuffer {
  pub(crate) fn new() -> Self {
    Self { antecedent_end_idx: 0, buffer: FilledBuffer::default(), current_end_idx: 0 }
  }

  pub(crate) const fn antecedent_end_idx(&self) -> usize {
    self.antecedent_end_idx
  }

  pub(crate) fn all(&self) -> &[u8] {
    self.buffer.all()
  }

  pub(crate) fn clear(&mut self) {
    let Self { antecedent_end_idx, buffer, current_end_idx } = self;
    *antecedent_end_idx = 0;
    buffer.clear();
    *current_end_idx = 0;
  }

  pub(crate) fn clear_if_following_is_empty(&mut self) {
    if !self.has_following() {
      self.clear();
    }
  }

  pub(crate) fn current(&self) -> &[u8] {
    let range = self.current_range();
    self.all().get(range).unwrap_or_default()
  }

  pub(crate) const fn current_end_idx(&self) -> usize {
    self.current_end_idx
  }

  pub(crate) const fn current_range(&self) -> Range<usize> {
    self.antecedent_end_idx()..self.current_end_idx()
  }

  pub(crate) fn following_end_idx(&self) -> usize {
    self.buffer.len()
  }

  pub(crate) fn following_len(&self) -> usize {
    self.following_end_idx().wrapping_sub(self.current_end_idx())
  }

  pub(crate) fn following_rest_mut(&mut self) -> &mut [u8] {
    let idx = self.current_end_idx();
    self.buffer.all_mut().get_mut(idx..).unwrap_or_default()
  }

  pub(crate) fn has_following(&self) -> bool {
    self.following_end_idx() > self.current_end_idx()
  }

  pub(crate) fn reserve(&mut self, additional: usize) -> crate::Result<()> {
    self.buffer.reserve(additional)
  }

  pub(crate) fn set_indices(
    &mut self,
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> crate::Result<()> {
    let [ant, cur, fol] = Self::indcs_from_lengths(antecedent_len, current_len, following_len);
    if fol > self.buffer.capacity() {
      return Err(crate::Error::InvalidPartitionedBufferBounds);
    }
    self.antecedent_end_idx = ant;
    self.current_end_idx = cur;
    self.buffer.set_len(fol);
    Ok(())
  }

  const fn indcs_from_lengths(
    antecedent_len: usize,
    current_len: usize,
    following_len: usize,
  ) -> [usize; 3] {
    let current_end_idx = antecedent_len.saturating_add(current_len);
    let following_end_idx = current_end_idx.saturating_add(following_len);
    [antecedent_len, current_end_idx, following_end_idx]
  }
}

#[cfg(any(feature = "mysql", feature = "postgres", feature = "web-socket"))]
impl PartitionedFilledBuffer {
  pub(crate) fn with_capacity(capacity: usize) -> crate::Result<Self> {
    Ok(Self {
      antecedent_end_idx: 0,
      buffer: FilledBuffer::with_capacity(capacity)?,
      current_end_idx: 0,
    })
  }
}

#[cfg(any(feature = "postgres", feature = "web-socket-handshake"))]
impl PartitionedFilledBuffer {
  pub(crate) fn suffix_writer(&mut self) -> crate::misc::SuffixWriterFbvm<'_> {
    crate::misc::SuffixWriterFbvm::new(self.following_end_idx(), self.buffer.vector_mut())
  }
}

#[cfg(feature = "web-socket")]
impl PartitionedFilledBuffer {
  pub(crate) fn current_mut(&mut self) -> &mut [u8] {
    let range = self.current_range();
    self.buffer.get_mut(range).unwrap_or_default()
  }

  pub(crate) fn current_rest_mut(&mut self) -> &mut [u8] {
    let idx = self.antecedent_end_idx();
    self.buffer.all_mut().get_mut(idx..).unwrap_or_default()
  }
}

#[cfg(feature = "web-socket-handshake")]
impl PartitionedFilledBuffer {
  pub(crate) fn all_mut(&mut self) -> &mut [u8] {
    self.buffer.all_mut()
  }

  pub(crate) fn following(&self) -> &[u8] {
    let idx = self.current_end_idx();
    self.all().get(idx..).unwrap_or_default()
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
