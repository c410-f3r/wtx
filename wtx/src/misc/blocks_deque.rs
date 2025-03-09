macro_rules! do_get {
  ($block:ident, $metadata:expr, $ptr:expr, $slice:ident, $($ref:tt)*) => {
    $block {
      data: {
        // SAFETY: `metadata` is always constructed with valid indices
        let pointer = unsafe { $ptr.add($metadata.begin) };
        // SAFETY: same as above
        unsafe { $($ref)* *ptr::$slice(pointer, $metadata.len) }
      },
      misc: $($ref)* $metadata.misc,
      range: $metadata.begin..$metadata.begin.wrapping_add($metadata.len)
    }
  }
}

macro_rules! get {
  ($metadata:expr, $ptr:expr) => {
    do_get!(BlockRef, $metadata, $ptr, slice_from_raw_parts, &)
  }
}

macro_rules! get_mut {
  ($metadata:expr, $ptr:expr) => {
    do_get!(BlockMut, $metadata, $ptr, slice_from_raw_parts_mut, &mut)
  }
}

mod block;
mod blocks_deque_builder;
mod metadata;
#[cfg(test)]
mod tests;

use crate::misc::Deque;
pub use block::Block;
pub use blocks_deque_builder::BlocksDequeBuilder;
use core::ptr;

/// [`Block`] composed by references.
type BlockRef<'bq, D, M> = Block<&'bq [D], &'bq M>;
/// [`Block`] composed by mutable references.
type BlockMut<'bq, D, M> = Block<&'bq mut [D], &'bq mut M>;

/// Errors of [`BlocksDeque`].
#[derive(Debug)]
pub enum BlocksDequeError {
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
  #[doc = doc_reserve_overflow!()]
  WithCapacityOverflow,
}

/// A circular buffer where elements are added in blocks that will never intersect boundaries.
#[derive(Debug)]
pub struct BlocksDeque<D, M> {
  data: Deque<D>,
  metadata: Deque<metadata::Metadata<M>>,
}

impl<D, M> BlocksDeque<D, M> {
  /// Creates a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Deque::new(), metadata: Deque::new() }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(blocks: usize, elements: usize) -> crate::Result<Self> {
    Ok(Self {
      data: Deque::with_capacity(elements)
        .map_err(|_err| BlocksDequeError::WithCapacityOverflow)?,
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

  /// See [`BlocksDequeBuilder`].
  #[inline]
  pub fn builder_back(&mut self) -> BlocksDequeBuilder<'_, D, M, true> {
    BlocksDequeBuilder::new(self)
  }

  /// See [`BlocksDequeBuilder`].
  #[inline]
  pub fn builder_front(&mut self) -> BlocksDequeBuilder<'_, D, M, false> {
    BlocksDequeBuilder::new(self)
  }

  /// Clears the queue, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    let Self { data, metadata } = self;
    data.clear();
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

  /// Provides a reference to a block at the given index.
  #[inline]
  pub fn get(&self, idx: usize) -> Option<BlockRef<'_, D, M>> {
    let metadata = self.metadata.get(idx)?;
    Some(get!(metadata, self.data.as_ptr()))
  }

  /// Mutable version of [`Self::get`].
  #[inline]
  pub fn get_mut(&mut self, idx: usize) -> Option<BlockMut<'_, D, M>> {
    let metadata = self.metadata.get_mut(idx)?;
    Some(get_mut!(metadata, self.data.as_ptr_mut()))
  }

  /// Returns a front-to-back iterator.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = BlockRef<'_, D, M>> {
    self.metadata.iter().map(|metadata| get!(metadata, self.data.as_ptr()))
  }

  /// Mutable version of [`Self::iter`].
  #[inline]
  pub fn iter_mut(&mut self) -> impl Iterator<Item = BlockMut<'_, D, M>> {
    let Self { data, metadata } = self;
    metadata
      .iter_mut()
      .map(move |elem| do_get!(BlockMut, elem, data.as_ptr_mut(), slice_from_raw_parts_mut, &mut))
  }

  /// Removes the last element from the queue and returns it, or `None` if it is empty.
  #[inline]
  pub fn pop_back(&mut self) -> Option<M> {
    let metadata = self.metadata.pop_back()?;
    self.data.truncate_back(self.elements_len().wrapping_sub(metadata.len));
    Some(metadata.misc)
  }

  /// Removes the first element and returns it, or [`Option::None`] if the queue is empty.
  #[inline]
  pub fn pop_front(&mut self) -> Option<M> {
    let metadata = self.metadata.pop_front()?;
    self.data.truncate_front(self.elements_len().wrapping_sub(metadata.len));
    Some(metadata.misc)
  }

  /// Appends a block to the end of the queue.
  #[inline]
  pub fn push_back_from_copyable_data<'data, I>(&mut self, data: I, misc: M) -> crate::Result<()>
  where
    D: Copy + 'data,
    I: IntoIterator<Item = &'data [D]>,
    I::IntoIter: Clone,
  {
    let total_data_len = self
      .data
      .extend_back_from_copyable_slices(data)
      .map_err(|_err| BlocksDequeError::PushOverflow)?;
    let begin = self.data.tail().wrapping_sub(total_data_len);
    self
      .metadata
      .push_back(metadata::Metadata { begin, len: total_data_len, misc })
      .map_err(|_err| BlocksDequeError::PushOverflow)?;
    Ok(())
  }

  /// Prepends a block to the queue.
  #[inline]
  pub fn push_front_from_coyable_data<'data, I>(&mut self, data: I, misc: M) -> crate::Result<()>
  where
    D: Copy + 'data,
    I: IntoIterator<Item = &'data [D]>,
    I::IntoIter: Clone,
  {
    let (total_data_len, head_shift) = self
      .data
      .extend_front_from_copyable_slices(data)
      .map_err(|_err| BlocksDequeError::PushOverflow)?;
    self
      .metadata
      .push_front(metadata::Metadata { begin: self.data.head(), len: total_data_len, misc })
      .map_err(|_err| BlocksDequeError::PushOverflow)?;
    self.adjust_metadata(head_shift, 1);
    Ok(())
  }

  /// Reserves capacity for at least additional more elements to be inserted in the given queue.
  #[inline(always)]
  pub fn reserve_front(&mut self, blocks: usize, elements: usize) -> crate::Result<()> {
    let _ = self.metadata.reserve_front(blocks).map_err(|_er| BlocksDequeError::ReserveOverflow)?;
    let n = self.data.reserve_front(elements).map_err(|_err| BlocksDequeError::ReserveOverflow)?;
    self.adjust_metadata(n, 0);
    Ok(())
  }

  // Only used in front operations
  #[inline]
  fn adjust_metadata(&mut self, head_shift: usize, skip: usize) {
    if head_shift > 0 {
      for metadata in self.metadata.iter_mut().skip(skip) {
        metadata.begin = metadata.begin.wrapping_add(head_shift);
      }
    }
  }
}

impl<D, M> Default for BlocksDeque<D, M> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}
