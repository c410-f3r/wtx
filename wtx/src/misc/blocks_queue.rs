macro_rules! as_slices {
  ($empty:expr, $ptr:ident, $slice:ident, $this:expr, $($ref:tt)*) => {{
    let len = $this.data.len();
    let rhs_len = $this.data.capacity().wrapping_sub($this.head);
    let ptr = $this.data.$ptr();
    if rhs_len < len {
      let lhs_len = len.wrapping_sub(rhs_len);
      // SAFETY: indices point to valid memory locations
      unsafe {
        (
          $($ref)* *ptr::$slice(ptr.add($this.head), rhs_len),
          $($ref)* *ptr::$slice(ptr, lhs_len),
        )
      }
    } else {
      // SAFETY: indices point to valid memory locations
      unsafe {
        ($($ref)* *ptr::$slice(ptr.add($this.head), len), $empty)
      }
    }
  }}
}

macro_rules! do_get {
  ($block:ident, $metadata:expr, $ptr:expr, $slice:ident, $($ref:tt)*) => {
    $block {
      // SAFETY: `metadata` is always constructed with valid indices
      data: unsafe {
        $($ref)* *ptr::$slice($ptr.add($metadata.begin), $metadata.len)
      },
      misc: $($ref)* $metadata.misc,
      range: {
        let end = $metadata.begin.wrapping_add($metadata.len);
        $metadata.begin..end
      }
    }
  }
}

use crate::misc::{
  queue_utils::{reserve, wrap_sub},
  Lease, Queue, SingleTypeStorage, Vector,
};
use core::{
  fmt::{Debug, Formatter},
  ops::Range,
  ptr,
};

pub(crate) type BlockRef<'bq, D, M> = Block<&'bq [D], &'bq M>;
pub(crate) type BlockMut<'bq, D, M> = Block<&'bq mut [D], &'bq mut M>;

/// Errors of [Queue].
#[derive(Debug)]
pub(crate) enum BlocksQueueError {
  #[doc = doc_single_elem_cap_overflow!()]
  PushFrontOverflow,
}

/// A circular buffer where elements are added in one-way blocks that will never intersect
/// boundaries.
pub(crate) struct BlocksQueue<D, M> {
  data: Vector<D>,
  head: usize,
  metadata: Queue<BlocksQueueMetadata<M>>,
  tail: usize,
}

impl<D, M> BlocksQueue<D, M>
where
  D: Copy,
  M: Copy,
{
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { data: Vector::new(), head: 0, metadata: Queue::new(), tail: 0 }
  }

  #[inline]
  pub(crate) fn with_capacity(blocks: usize, elements: usize) -> Self {
    Self {
      data: Vector::with_capacity(elements),
      head: 0,
      metadata: Queue::with_capacity(blocks),
      tail: 0,
    }
  }

  #[inline]
  pub(crate) fn as_slices(&self) -> (&[D], &[D]) {
    as_slices!(&[][..], as_ptr, slice_from_raw_parts, self, &)
  }

  #[cfg(test)]
  #[inline]
  pub(crate) fn blocks_capacity(&self) -> usize {
    self.metadata.capacity()
  }

  #[inline]
  pub(crate) fn blocks_len(&self) -> usize {
    self.metadata.len()
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { data, head, metadata, tail } = self;
    data.clear();
    *head = 0;
    *tail = 0;
    metadata.clear();
  }

  #[inline]
  pub(crate) fn elements_capacity(&self) -> usize {
    self.data.capacity()
  }

  #[inline]
  pub(crate) fn elements_len(&self) -> usize {
    self.data.len()
  }

  #[inline]
  pub(crate) fn first(&self) -> Option<BlockRef<'_, D, M>> {
    self.get(0)
  }

  #[inline]
  pub(crate) fn get(&self, idx: usize) -> Option<BlockRef<'_, D, M>> {
    Some(Self::do_get(&self.data, self.metadata.get(idx)?))
  }

  #[inline]
  pub(crate) fn get_mut(&mut self, idx: usize) -> Option<BlockMut<'_, D, M>> {
    Some(Self::do_get_mut(&mut self.data, self.metadata.get_mut(idx)?))
  }

  #[inline]
  pub(crate) fn iter(&self) -> impl Iterator<Item = BlockRef<'_, D, M>> {
    self.metadata.iter().map(|metadata| Self::do_get(&self.data, metadata))
  }

  #[inline]
  pub(crate) fn last(&self) -> Option<BlockRef<'_, D, M>> {
    self.get(self.data.len().checked_sub(1)?)
  }

  #[inline]
  pub(crate) fn pop_back(&mut self) -> Option<(M, &mut [D])> {
    let metadata = self.metadata.pop_back()?;
    if let Some(elem) = self.metadata.last() {
      self.tail = elem.begin.wrapping_add(elem.len);
    } else {
      self.head = 0;
      self.tail = 0;
    };
    // SAFETY: structure is not empty
    let slice = unsafe {
      self.data.set_len(self.data.len().wrapping_sub(metadata.len));
      &mut *ptr::slice_from_raw_parts_mut(self.data.as_mut_ptr().add(metadata.begin), metadata.len)
    };
    Some((metadata.misc, slice))
  }

  #[inline]
  pub(crate) fn pop_front(&mut self) -> Option<(M, &mut [D])> {
    let metadata = self.metadata.pop_front()?;
    if let Some(elem) = self.metadata.first() {
      self.head = elem.begin;
    } else {
      self.head = 0;
      self.tail = 0;
    };
    // SAFETY: structure is not empty
    let slice = unsafe {
      self.data.set_len(self.data.len().wrapping_sub(metadata.len));
      &mut *ptr::slice_from_raw_parts_mut(self.data.as_mut_ptr().add(metadata.begin), metadata.len)
    };
    Some((metadata.misc, slice))
  }

  #[inline]
  pub(crate) fn push_front<const N: usize>(
    &mut self,
    data: [&[D]; N],
    misc: M,
  ) -> crate::Result<()> {
    let mut len: usize = 0;
    for elem in data {
      len = len.wrapping_add(elem.len());
    }
    let mut tail = self.tail;
    let (left_free, right_free) = self.free(|| {
      tail = self.data.capacity();
    });
    let head = match (left_free >= len, right_free >= len) {
      (true, _) => self.head_lhs(len),
      (false, true) => self.head_rhs(len),
      (false, false) => return Err(BlocksQueueError::PushFrontOverflow.into()),
    };
    self.metadata.push_front(BlocksQueueMetadata { begin: head, len, misc })?;
    self.head = head;
    self.tail = tail;
    // SAFETY: indices point to valid memory locations
    unsafe {
      let mut start = self.head;
      for value in data {
        ptr::copy_nonoverlapping(value.as_ptr(), self.data.as_mut_ptr().add(start), value.len());
        start = start.wrapping_add(value.len());
      }
      self.data.set_len(self.data.len().wrapping_add(len));
    }
    Ok(())
  }

  #[inline(always)]
  pub(crate) fn reserve(&mut self, blocks: usize, elements: usize) {
    let is_not_wrapped = !self.is_wrapped();
    let prev_head = self.head;
    let diff_opt = reserve(elements, &mut self.data, &mut self.head);
    self.metadata.reserve(blocks);
    if let Some(diff) = diff_opt {
      if is_not_wrapped {
        self.tail = self.tail.wrapping_add(diff);
      }
      let mut iter = self.metadata.iter_mut();
      for elem in iter.by_ref() {
        if elem.begin >= prev_head {
          elem.begin = elem.begin.wrapping_add(diff);
          break;
        }
      }
      for elem in iter {
        elem.begin = elem.begin.wrapping_add(diff);
      }
    }
  }

  #[inline]
  fn do_get<'this>(
    data: &'this Vector<D>,
    metadata: &'this BlocksQueueMetadata<M>,
  ) -> BlockRef<'this, D, M> {
    do_get!(BlockRef, metadata, data.as_ptr(), slice_from_raw_parts, &)
  }

  #[inline]
  fn do_get_mut<'this>(
    data: &'this mut Vector<D>,
    metadata: &'this mut BlocksQueueMetadata<M>,
  ) -> BlockMut<'this, D, M> {
    do_get!(BlockMut, metadata, data.as_mut_ptr(), slice_from_raw_parts_mut, &mut)
  }

  #[inline]
  fn free(&self, cb: impl FnOnce()) -> (usize, usize) {
    self.wrap_logic(
      || {
        cb();
        (self.elements_capacity(), 0)
      },
      || (self.head, self.data.capacity().wrapping_sub(self.tail)),
      || (self.head.wrapping_sub(self.tail), 0),
    )
  }

  #[inline]
  fn head_lhs(&self, len: usize) -> usize {
    wrap_sub(self.data.capacity(), self.head, len)
  }

  #[inline]
  fn head_rhs(&self, len: usize) -> usize {
    self.data.capacity().wrapping_sub(len)
  }

  #[inline]
  fn is_wrapped(&self) -> bool {
    self.wrap_logic(|| false, || false, || true)
  }

  #[inline]
  fn wrap_logic<R>(
    &self,
    empty: impl FnOnce() -> R,
    contiguous: impl FnOnce() -> R,
    wraps: impl FnOnce() -> R,
  ) -> R {
    if self.head == 0 && self.tail == 0 {
      empty()
    } else if self.head < self.tail {
      contiguous()
    } else {
      wraps()
    }
  }
}

impl<D, M> Debug for BlocksQueue<D, M>
where
  D: Copy + Debug,
  M: Copy + Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    let (lhs, rhs) = self.as_slices();
    f.debug_struct("BlocksQueue").field("lhs", &lhs).field("rhs", &rhs).finish()
  }
}

impl<D, M> Default for BlocksQueue<D, M>
where
  D: Copy,
  M: Copy,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Block<D, M>
where
  D: Lease<[D::Item]> + SingleTypeStorage,
{
  pub(crate) data: D,
  pub(crate) misc: M,
  pub(crate) range: Range<usize>,
}

#[derive(Clone, Copy, Debug)]
struct BlocksQueueMetadata<M> {
  pub(crate) begin: usize,
  pub(crate) len: usize,
  pub(crate) misc: M,
}

// H = Head (Inclusive)
// LF = Left Free
// LO = Left Occupied
// RF = Right Free
// RO = Right Occupied
// T = Tail (Exclusive)
#[cfg(test)]
mod tests {
  use crate::misc::{blocks_queue::BlockRef, BlocksQueue};

  // [. . . . . . . .]: Empty - (LF=8, LO=0,RF=0, RO=0) - (H=0, T=0)
  // [. . . . . . . H]: Push front - (LF=7, LO=0, RF=0, RO=1) - (H=7, T=8)
  // [. . . . . H * *]: Push front - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
  // [. . H * * * * *]: Push front - (LF=2, LO=0, RF=0, RO=6) - (H=2, T=8)
  // [H * * * * * * *]: Push front - (LF=0, LO=0, RF=0, RO=8) - (H=0, T=8)
  // [H * * * * * * .]: Pop back - (LF=0, LO=0, RF=2, RO=6) - (H=0, T=7)
  // [* * * * * * * H]: Push front - (LF=0, LO=7, RF=0, RO=0) - (H=7, T=7)
  // [* * * * * . . H]: Pop back - (LF=2, LO=5, RF=0, RO=1) - (H=7, T=5)
  // [* * . . . . . H]: Pop back - (LF=5, LO=2, RF=0, RO=1) - (H=7, T=2)
  // [* * . . H * * *]: Push front - (LF=2, LO=2, RF=0, RO=4) - (H=4, T=2)
  // [* * H * * * * *]: Push front - (LF=0, LO=2, RF=0, RO=6) - (H=2, T=2)
  // [. . H * * * * *]: Pop back - (LF=2, LO=0, RF=0, RO=6) - (H=2, T=8)
  // [. . H * * * * .]: Pop back - (LF=2, LO=0, RF=1, RO=5) - (H=2, T=7)
  // [. . H * . . . .]: Pop back - (LF=1, LO=0, RF=4, RO=3) - (H=2, T=4)
  // [. H * * . . . .]: Push front - (LF=1, LO=0, RF=4, RO=3) - (H=2, T=4)
  // [H * * * . . . .]: Push front - (LF=0, LO=0, RF=4, RO=4) - (H=0, T=4)
  // [H . . . . . . .]: Pop back - (LF=0, LO=0, RF=7, RO=1) - (H=0, T=1)
  // [. . . . . . . .]: Pop back - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
  #[test]
  fn pop_back() {
    let mut q = BlocksQueue::with_capacity(4, 8);
    check_state(&q, 0, 0, 0, 0);

    q.push_front([&[1]], ()).unwrap();
    check_state(&q, 1, 1, 7, 8);

    q.push_front([&[2, 3]], ()).unwrap();
    check_state(&q, 2, 3, 5, 8);

    q.push_front([&[4, 5], &[6]], ()).unwrap();
    check_state(&q, 3, 6, 2, 8);

    q.push_front([&[7, 8]], ()).unwrap();
    check_state(&q, 4, 8, 0, 8);

    let _ = q.pop_back();
    check_state(&q, 3, 7, 0, 7);

    q.push_front([&[9]], ()).unwrap();
    check_state(&q, 4, 8, 7, 7);

    let _ = q.pop_back();
    check_state(&q, 3, 6, 7, 5);

    let _ = q.pop_back();
    check_state(&q, 2, 3, 7, 2);

    q.push_front([&[10], &[11, 12]], ()).unwrap();
    check_state(&q, 3, 6, 4, 2);

    q.push_front([&[13, 14]], ()).unwrap();
    check_state(&q, 4, 8, 2, 2);

    let _ = q.pop_back();
    check_state(&q, 3, 6, 2, 8);

    let _ = q.pop_back();
    check_state(&q, 2, 5, 2, 7);

    let _ = q.pop_back();
    check_state(&q, 1, 2, 2, 4);

    q.push_front([&[15]], ()).unwrap();
    check_state(&q, 2, 3, 1, 4);

    q.push_front([&[16]], ()).unwrap();
    check_state(&q, 3, 4, 0, 4);

    let _ = q.pop_back();
    check_state(&q, 2, 2, 0, 2);

    let _ = q.pop_back();
    check_state(&q, 1, 1, 0, 1);

    let _ = q.pop_back();
    check_state(&q, 0, 0, 0, 0);
  }

  // [. . . . . . . .]: Empty - (LF=8, LO=0,RF=0, RO=0) - (H=0, T=0)
  // [. . . . . H * *]: Push front - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
  // [H * * * * * * *]: Push front - (LF=0, LO=0, RF=0, RO=8) - (H=0, T=8)
  // [. . . . . H * *]: Push front - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
  // [. . . . . . . .]: Pop back - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
  #[test]
  fn pop_front() {
    let mut q = BlocksQueue::with_capacity(2, 8);
    check_state(&q, 0, 0, 0, 0);

    q.push_front([&[1, 2, 3]], ()).unwrap();
    check_state(&q, 1, 3, 5, 8);

    q.push_front([&[4, 5], &[6, 7, 8]], ()).unwrap();
    check_state(&q, 2, 8, 0, 8);

    let _ = q.pop_front();
    check_state(&q, 1, 3, 5, 8);

    let _ = q.pop_front();
    check_state(&q, 0, 0, 0, 0);
  }

  // []: Empty - (LF=0, LO=0,RF=0, RO=0) - (H=0, T=0)
  // [H * * *]: Push front - (LF=0, LO=0, RF=0, RO=4) - (H=0, T=4)
  #[test]
  fn push_reserve_and_push() {
    let mut q = BlocksQueue::new();
    q.reserve(1, 4);
    q.push_front([&[0, 1, 2, 3]], ()).unwrap();
    check_state(&q, 1, 4, 0, 4);
    assert_eq!(q.get(0), Some(BlockRef { data: &[0, 1, 2, 3], misc: &(), range: 0..4 }));
    assert_eq!(q.get(1), None);
    q.reserve(1, 6);
    q.push_front([&[4, 5, 6, 7, 8, 9]], ()).unwrap();
    check_state(&q, 2, 10, 0, 10);
    assert_eq!(q.get(0), Some(BlockRef { data: &[4, 5, 6, 7, 8, 9], misc: &(), range: 0..6 }));
    assert_eq!(q.get(1), Some(BlockRef { data: &[0, 1, 2, 3], misc: &(), range: 6..10 }));
    assert_eq!(q.get(2), None);
  }

  #[test]
  fn reserve() {
    let mut queue = BlocksQueue::<i32, ()>::new();
    assert_eq!(queue.blocks_capacity(), 0);
    assert_eq!(queue.elements_capacity(), 0);
    queue.reserve(5, 10);
    assert_eq!(queue.blocks_capacity(), 5);
    assert_eq!(queue.elements_capacity(), 10);
  }

  // [. . . . . H * * ]: Pop back - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
  // [. . . . . . . . ]: Pop back - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
  #[test]
  fn wrap_pop_back() {
    let mut q = wrap_initial();
    let _ = q.pop_back();
    let _ = q.pop_back();
    check_state(&q, 1, 3, 5, 8);
    assert_eq!(q.get(0).unwrap().data, &[1, 2, 3]);
    let _ = q.pop_back();
    check_state(&q, 0, 0, 0, 0);
  }

  // [. . H * . . . . ]: Pop front - (LF=2, LO=0, RF=4, RO=0) - (H=2, T=4)
  // [. . . . . . . . ]: Pop front - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
  #[test]
  fn wrap_pop_front() {
    let mut q = wrap_initial();
    let _ = q.pop_front();
    check_state(&q, 2, 2, 2, 4);
    assert_eq!(q.get(0).unwrap().data, &[0]);
    assert_eq!(q.get(1).unwrap().data, &[0]);
    let _ = q.pop_front();
    let _ = q.pop_front();
    check_state(&q, 0, 0, 0, 0);
  }

  // [. . . . . . . . ]: Empty - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
  // [. . H * * * * * ]: Push front - (LF=2, LO=0, RF=0, RO=6) - (H=2, T=8)
  // [. . H * . . . . ]: Pop back - (LF=2, LO=0, RF=4, RO=0) - (H=2, T=4)
  // [. . * * . H * * ]: Push front - (LF=1, LO=2, RF=0, RO=3) - (H=5, T=4)
  fn wrap_initial() -> BlocksQueue<i32, ()> {
    let mut q = BlocksQueue::with_capacity(6, 8);
    check_state(&q, 0, 0, 0, 0);
    for _ in 0..6 {
      q.push_front([&[0]], ()).unwrap();
    }
    check_state(&q, 6, 6, 2, 8);
    for idx in 0..6 {
      assert_eq!(q.get(idx).unwrap().data, &[0]);
    }
    let _ = q.pop_back();
    let _ = q.pop_back();
    let _ = q.pop_back();
    let _ = q.pop_back();
    check_state(&q, 2, 2, 2, 4);
    assert_eq!(q.get(0).unwrap().data, &[0]);
    assert_eq!(q.get(1).unwrap().data, &[0]);
    q.push_front([&[1, 2, 3]], ()).unwrap();
    check_state(&q, 3, 5, 5, 4);
    assert_eq!(q.get(0).unwrap().data, &[1, 2, 3]);
    assert_eq!(q.get(1).unwrap().data, &[0]);
    assert_eq!(q.get(2).unwrap().data, &[0]);
    q
  }

  #[track_caller]
  fn check_state(
    q: &BlocksQueue<i32, ()>,
    blocks_len: usize,
    elements_len: usize,
    head: usize,
    tail: usize,
  ) {
    assert_eq!(q.blocks_len(), blocks_len);
    assert_eq!(q.elements_len(), elements_len);
    assert_eq!(q.head, head);
    assert_eq!(q.tail, tail);
  }
}
