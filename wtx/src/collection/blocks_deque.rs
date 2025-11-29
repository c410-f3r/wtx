#![expect(clippy::ref_patterns, reason = "false-positive")]

macro_rules! do_get {
  ($block:ident, $data:expr, $metadata:expr, $logical_begin:expr, $ptr:ident, $slice:ident, $($ref:tt)*) => {{
    let metadata = $metadata;
    let data = $($ref)* *$data;
    let len = metadata.len;
    let relative_offset = metadata.offset.wrapping_sub($logical_begin);

    let head = data.head();
    let capacity = data.capacity();
    let physical_begin = head.wrapping_add(relative_offset).checked_rem(capacity);

    $block {
      data: if let Some(elem) = physical_begin {
        // SAFETY: `metadata` is always constructed with valid indices
        let pointer = unsafe { data.$ptr().add(elem) };
        // SAFETY: same as above
        unsafe { slice::$slice(pointer, len) }
      } else {
        $($ref)* []
      },
      misc: $($ref)* metadata.misc,
      range: {
        relative_offset..relative_offset.wrapping_add(len)
      }
    }
  }}
}

macro_rules! get {
  ($data:expr, $logical_begin:expr, $metadata:expr) => {
    do_get!(BlockRef, $data, $metadata, $logical_begin, as_ptr, from_raw_parts, &)
  }
}

macro_rules! get_mut {
  ($data:expr, $logical_begin:expr, $metadata:expr) => {
    do_get!(BlockMut, $data, $metadata, $logical_begin, as_ptr_mut, from_raw_parts_mut, &mut)
  }
}

mod block;
mod metadata;
#[cfg(test)]
mod tests;

use crate::collection::{Deque, ExpansionTy, TryExtend};
pub use block::Block;
use core::slice;

/// [`Block`] composed by references.
type BlockRef<'bq, D, M> = Block<&'bq [D], &'bq M>;
/// [`Block`] composed by mutable references.
type BlockMut<'bq, D, M> = Block<&'bq mut [D], &'bq mut M>;

/// Errors of [`BlocksDeque`].
#[derive(Debug)]
pub enum BlocksDequeError {
  /// The provided index does not point to valid internal data
  OutOfBoundsIndex,
  #[doc = doc_single_elem_cap_overflow!()]
  PushBackOverflow,
  #[doc = doc_single_elem_cap_overflow!()]
  PushFrontDataOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
  #[doc = doc_reserve_overflow!()]
  WithCapacityOverflow,
}

/// A circular buffer where elements are added in blocks that will never intersect boundaries.
#[derive(Debug)]
pub struct BlocksDeque<D, M> {
  data: Deque<D>,
  logical_begin: usize,
  metadata: Deque<metadata::Metadata<M>>,
}

impl<D, M> BlocksDeque<D, M> {
  /// Creates a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Deque::new(), logical_begin: 0, metadata: Deque::new() }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(blocks: usize, elements: usize) -> crate::Result<Self> {
    Ok(Self {
      data: Deque::with_capacity(elements)
        .map_err(|_err| BlocksDequeError::WithCapacityOverflow)?,
      logical_begin: 0,
      metadata: Deque::with_capacity(blocks)
        .map_err(|_err| BlocksDequeError::WithCapacityOverflow)?,
    })
  }

  /// Constructs a new, empty instance with the exact specified capacity.
  #[inline]
  pub fn with_exact_capacity(blocks: usize, elements: usize) -> crate::Result<Self> {
    Ok(Self {
      data: Deque::with_exact_capacity(elements)
        .map_err(|_err| BlocksDequeError::WithCapacityOverflow)?,
      logical_begin: 0,
      metadata: Deque::with_exact_capacity(blocks)
        .map_err(|_err| BlocksDequeError::WithCapacityOverflow)?,
    })
  }

  /// Returns a pair of slices which contain, in order, the contents of the queue.
  #[inline]
  pub fn as_slices(&self) -> (&[D], &[D]) {
    self.data.as_slices()
  }

  /// Returns the number of blocks the queue can hold without reallocating.
  #[inline]
  pub fn blocks_capacity(&self) -> usize {
    self.metadata.capacity()
  }

  /// Returns the number of blocks.
  #[inline]
  pub fn blocks_len(&self) -> usize {
    self.metadata.len()
  }

  /// Clears the queue, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    let Self { data, logical_begin, metadata } = self;
    data.clear();
    *logical_begin = 0;
    metadata.clear();
  }

  /// Returns the number of elements the queue can hold without reallocating.
  #[inline]
  pub fn elements_capacity(&self) -> usize {
    self.data.capacity()
  }

  /// Returns the number of elements.
  #[inline]
  pub fn elements_len(&self) -> usize {
    self.data.len()
  }

  /// Appends elements to the back of the instance so that the current length is equal to `et`.
  #[inline]
  pub fn expand_back(&mut self, et: ExpansionTy, misc: M, value: D) -> crate::Result<usize>
  where
    D: Clone,
  {
    let Self { data, logical_begin, metadata } = self;
    let old_len = data.len();
    let total_data_len =
      data.expand_back(et, value).map_err(|_err| BlocksDequeError::PushBackOverflow)?;
    let new_offset = logical_begin.wrapping_add(old_len);
    metadata
      .push_back(metadata::Metadata { offset: new_offset, len: total_data_len, misc })
      .map_err(|_err| BlocksDequeError::PushBackOverflow)?;
    Ok(total_data_len)
  }

  /// Provides a reference to a block at the given index.
  #[inline]
  pub fn get(&self, idx: usize) -> Option<BlockRef<'_, D, M>> {
    let Self { ref data, logical_begin, ref metadata } = *self;
    let local_metadata = metadata.get(idx)?;
    Some(get!(data, logical_begin, local_metadata))
  }

  /// Mutable version of [`Self::get`].
  #[inline]
  pub fn get_mut(&mut self, idx: usize) -> Option<BlockMut<'_, D, M>> {
    let Self { ref mut data, logical_begin, ref mut metadata } = *self;
    let local_metadata = metadata.get_mut(idx)?;
    Some(get_mut!(data, logical_begin, local_metadata))
  }

  /// Returns a front-to-back iterator.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = BlockRef<'_, D, M>> {
    let Self { ref data, logical_begin, ref metadata } = *self;
    metadata.iter().map(move |elem| get!(data, logical_begin, elem))
  }

  /// Mutable version of [`Self::iter`].
  #[inline]
  pub fn iter_mut(&mut self) -> impl Iterator<Item = BlockMut<'_, D, M>> {
    let Self { ref mut data, logical_begin, ref mut metadata } = *self;
    metadata.iter_mut().map(move |elem| get_mut!(data, logical_begin, elem))
  }

  /// Removes the last element from the queue and returns it, or `None` if it is empty.
  #[inline]
  pub fn pop_back(&mut self) -> Option<M> {
    let Self { data, logical_begin: _, metadata } = self;
    let local_metadata = metadata.pop_back()?;
    data.truncate_back(data.len().wrapping_sub(local_metadata.len));
    Some(local_metadata.misc)
  }

  /// Appends a block to the end of the queue.
  #[inline]
  pub fn push_back_from_copyable_data<'data, I>(
    &mut self,
    local_data: I,
    misc: M,
  ) -> crate::Result<()>
  where
    D: Copy + 'data,
    I: IntoIterator<Item = &'data [D]>,
    I::IntoIter: Clone,
  {
    let Self { data, logical_begin, metadata } = self;
    let offset = logical_begin.wrapping_add(data.len());
    let total_data_len = data
      .extend_back_from_copyable_slices(local_data)
      .map_err(|_err| BlocksDequeError::PushBackOverflow)?;
    metadata
      .push_back(metadata::Metadata { len: total_data_len, misc, offset })
      .map_err(|_err| BlocksDequeError::PushBackOverflow)?;
    Ok(())
  }

  /// See [`Self::pop_back`]. Transfers elements to `buffer` instead of dropping them.
  #[inline]
  pub fn pop_back_to_buffer<B>(&mut self, buffer: &mut B) -> Option<crate::Result<M>>
  where
    B: TryExtend<[D; 1]>,
  {
    let Self { ref mut data, logical_begin: _, ref mut metadata } = *self;
    let local_metadata = metadata.pop_back()?;
    let new_len = data.len().wrapping_sub(local_metadata.len);
    if let Err(err) = data.truncate_back_to_buffer(buffer, new_len) {
      return Some(Err(err));
    }
    Some(Ok(local_metadata.misc))
  }

  /// Removes the first element and returns it, or [`Option::None`] if the queue is empty.
  #[inline]
  pub fn pop_front(&mut self) -> Option<M> {
    let Self { data, logical_begin, metadata } = self;
    let local_metadata = metadata.pop_front()?;
    data.truncate_front(data.len().wrapping_sub(local_metadata.len));
    *logical_begin = logical_begin.wrapping_add(local_metadata.len);
    Some(local_metadata.misc)
  }

  /// Prepends a block to the queue.
  #[inline]
  pub fn push_front_from_copyable_data<'data, I>(
    &mut self,
    local_data: I,
    misc: M,
  ) -> crate::Result<()>
  where
    D: Copy + 'data,
    I: IntoIterator<Item = &'data [D]>,
    I::IntoIter: Clone,
  {
    let Self { data, logical_begin, metadata } = self;
    let total_data_len = data
      .extend_front_from_copyable_slices(local_data)
      .map_err(|_err| BlocksDequeError::PushFrontDataOverflow)?;
    *logical_begin = logical_begin.wrapping_sub(total_data_len);
    metadata
      .push_front(metadata::Metadata { offset: *logical_begin, len: total_data_len, misc })
      .map_err(|_err| BlocksDequeError::PushFrontDataOverflow)?;
    Ok(())
  }

  /// See [`Self::pop_front`]. Transfers elements to `buffer` instead of dropping them.
  #[inline]
  pub fn pop_front_to_buffer<B>(&mut self, buffer: &mut B) -> Option<crate::Result<M>>
  where
    B: TryExtend<[D; 1]>,
  {
    let Self { data, logical_begin, metadata } = self;
    let local_metadata = metadata.pop_front()?;
    let new_len = data.len().wrapping_sub(local_metadata.len);
    if let Err(err) = data.truncate_front_to_buffer(buffer, new_len) {
      return Some(Err(err));
    }
    *logical_begin = logical_begin.wrapping_add(local_metadata.len);
    Some(Ok(local_metadata.misc))
  }

  /// Reserves capacity for at least additional more elements to be inserted in the given queue.
  #[inline(always)]
  pub fn reserve_front(&mut self, blocks: usize, elements: usize) -> crate::Result<()> {
    let Self { data, logical_begin: _, metadata } = self;
    let _ = metadata.reserve_front(blocks).map_err(|_err| BlocksDequeError::ReserveOverflow)?;
    let _ = data.reserve_front(elements).map_err(|_err| BlocksDequeError::ReserveOverflow)?;
    Ok(())
  }
}

impl<D, M> Default for BlocksDeque<D, M> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
