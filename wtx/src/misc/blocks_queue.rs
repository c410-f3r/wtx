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

#[cfg(test)]
mod tests;

use crate::misc::Queue;
use core::{ops::Range, ptr};

/// [`Block`] composed by references.
type BlockRef<'bq, D, M> = Block<&'bq [D], &'bq M>;
/// [`Block`] composed by mutable references.
type BlockMut<'bq, D, M> = Block<&'bq mut [D], &'bq mut M>;

/// Errors of [`BlocksQueue`].
#[derive(Debug)]
pub enum BlocksQueueError {
  #[doc = doc_single_elem_cap_overflow!()]
  PushOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
  #[doc = doc_reserve_overflow!()]
  WithCapacityOverflow,
}

/// A circular buffer where elements are added in one-way blocks that will never intersect
/// boundaries.
#[derive(Debug)]
pub struct BlocksQueue<D, M> {
  data: Queue<D>,
  metadata: Queue<Metadata<M>>,
}

impl<D, M> BlocksQueue<D, M> {
  /// Creates a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Queue::new(), metadata: Queue::new() }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(blocks: usize, elements: usize) -> Result<Self, BlocksQueueError> {
    Ok(Self {
      data: Queue::with_capacity(elements)
        .map_err(|_err| BlocksQueueError::WithCapacityOverflow)?,
      metadata: Queue::with_capacity(blocks)
        .map_err(|_err| BlocksQueueError::WithCapacityOverflow)?,
    })
  }

  /// Constructs a new, empty instance with the exact specified capacity.
  #[inline]
  pub fn with_exact_capacity(blocks: usize, elements: usize) -> Result<Self, BlocksQueueError> {
    Ok(Self {
      data: Queue::with_exact_capacity(elements)
        .map_err(|_err| BlocksQueueError::WithCapacityOverflow)?,
      metadata: Queue::with_exact_capacity(blocks)
        .map_err(|_err| BlocksQueueError::WithCapacityOverflow)?,
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

  /// Returns the last block.
  #[inline]
  pub fn last(&self) -> Option<BlockRef<'_, D, M>> {
    self.get(self.data.len().checked_sub(1)?)
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

  /// Prepends an block to the queue.
  #[inline]
  pub fn push_front<'data, I>(&mut self, data: I, misc: M) -> Result<(), BlocksQueueError>
  where
    D: Copy + 'data,
    I: IntoIterator<Item = &'data [D]>,
    I::IntoIter: Clone,
  {
    let (total_data_len, head_shift) = self
      .data
      .extend_front_from_copyable_slices(data)
      .map_err(|_err| BlocksQueueError::PushOverflow)?;
    self
      .metadata
      .push_front(Metadata { begin: self.data.head(), len: total_data_len, misc })
      .map_err(|_err| BlocksQueueError::PushOverflow)?;
    self.adjust_metadata(head_shift, 1);
    Ok(())
  }

  /// Reserves capacity for at least additional more elements to be inserted in the given queue.
  #[inline(always)]
  pub fn reserve_front(&mut self, blocks: usize, elements: usize) -> Result<(), BlocksQueueError> {
    let _ = self.metadata.reserve_front(blocks).map_err(|_er| BlocksQueueError::ReserveOverflow)?;
    let n = self.data.reserve_front(elements).map_err(|_err| BlocksQueueError::ReserveOverflow)?;
    self.adjust_metadata(n, 0);
    Ok(())
  }

  #[inline]
  fn adjust_metadata(&mut self, head_shift: usize, skip: usize) {
    if head_shift > 0 {
      for metadata in self.metadata.iter_mut().skip(skip) {
        metadata.begin = metadata.begin.wrapping_add(head_shift);
      }
    }
  }
}

impl<D, M> Default for BlocksQueue<D, M> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// Block
#[derive(Debug, PartialEq)]
pub struct Block<D, M> {
  /// Opaque data
  pub data: D,
  /// Miscellaneous
  pub misc: M,
  /// Range
  pub range: Range<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Metadata<M> {
  begin: usize,
  len: usize,
  misc: M,
}
