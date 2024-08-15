macro_rules! as_slices {
  ($empty:expr, $ptr:ident, $slice:ident, $this:expr, $($ref:tt)*) => {{
    let len = $this.data.len();
    // SAFETY: `this.head` will never be greater than capacity
    let rhs_len = unsafe { $this.data.capacity().unchecked_sub($this.head) };
    let ptr = $this.data.$ptr();
    // SAFETY: inner data is expected to point to valid memory
    let added_ptr = unsafe { ptr.add($this.head) };
    if rhs_len < len {
      // SAFETY: `ìf` check ensures bounds
      let lhs = unsafe { $($ref)* *ptr::$slice(ptr, len.wrapping_sub(rhs_len)) };
      // SAFETY: `ìf` check ensures bounds
      let rhs = unsafe { $($ref)* *ptr::$slice(added_ptr, rhs_len) };
      (lhs, rhs)
    } else {
      // SAFETY: `ìf` check ensures bounds
      let lhs = unsafe { $($ref)* *ptr::$slice(added_ptr, len) };
      (lhs, $empty)
    }
  }}
}

use crate::misc::{
  queue_utils::{reserve, wrap_add, wrap_sub},
  Vector,
};
use core::{
  fmt::{Debug, Formatter},
  ptr,
};

/// Errors of [Queue].
#[derive(Debug)]
pub enum QueueError {
  #[doc = doc_single_elem_cap_overflow!()]
  PushFrontOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
  #[doc = doc_reserve_overflow!()]
  WithCapacityOverflow,
}

/// A circular buffer.
pub struct Queue<D> {
  data: Vector<D>,
  head: usize,
}

impl<D> Queue<D> {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { data: Vector::new(), head: 0 }
  }

  #[inline]
  pub(crate) fn with_capacity(cap: usize) -> Result<Self, QueueError> {
    Ok(Self {
      data: Vector::with_capacity(cap).map_err(|_err| QueueError::WithCapacityOverflow)?,
      head: 0,
    })
  }

  #[inline]
  pub(crate) fn as_slices(&self) -> (&[D], &[D]) {
    as_slices!(&[][..], as_ptr, slice_from_raw_parts, self, &)
  }

  #[inline]
  pub(crate) fn as_slices_mut(&mut self) -> (&mut [D], &mut [D]) {
    as_slices!(&mut [][..], as_mut_ptr, slice_from_raw_parts_mut, self, &mut)
  }

  #[cfg(test)]
  #[inline]
  pub(crate) fn capacity(&self) -> usize {
    self.data.capacity()
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    self.head = 0;
    self.data.clear();
  }

  #[inline]
  pub(crate) fn first(&self) -> Option<&D> {
    self.get(0)
  }

  #[inline]
  pub(crate) fn get(&self, mut idx: usize) -> Option<&D> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` points to valid memory
    let rslt = unsafe { self.data.as_ptr().add(idx) };
    // SAFETY: `idx` points to valid memory
    unsafe { Some(&*rslt) }
  }

  #[inline]
  pub(crate) fn get_mut(&mut self, mut idx: usize) -> Option<&mut D> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` points to valid memory
    let rslt = unsafe { self.data.as_mut_ptr().add(idx) };
    // SAFETY: `idx` points to valid memory
    unsafe { Some(&mut *rslt) }
  }

  #[inline]
  pub(crate) fn iter(&self) -> impl Iterator<Item = &D> {
    let (lhs, rhs) = self.as_slices();
    rhs.iter().chain(lhs)
  }

  #[inline]
  pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut D> {
    let (lhs, rhs) = self.as_slices_mut();
    rhs.iter_mut().chain(lhs)
  }

  #[inline]
  pub(crate) fn last(&self) -> Option<&D> {
    self.get(self.len().checked_sub(1)?)
  }

  #[inline]
  pub(crate) fn len(&self) -> usize {
    self.data.len()
  }

  #[inline]
  pub(crate) fn pop_back(&mut self) -> Option<D> {
    let new_len = self.data.len().checked_sub(1)?;
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    let idx = wrap_add(self.data.capacity(), self.head, new_len);
    // SAFETY: `idx` points to valid memory
    let src = unsafe { self.data.as_mut_ptr().add(idx) };
    // SAFETY: `src` points to valid memory
    unsafe { Some(ptr::read(src)) }
  }

  #[inline]
  pub(crate) fn pop_front(&mut self) -> Option<D> {
    let new_len = self.data.len().checked_sub(1)?;
    let prev_head = self.head;
    self.head = wrap_add(self.data.capacity(), self.head, 1);
    // SAFETY: `prev_head` points to valid memory
    let src = unsafe { self.data.as_mut_ptr().add(prev_head) };
    // SAFETY: `src` points to valid memory
    let rslt = unsafe { ptr::read(src) };
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    Some(rslt)
  }
}

impl<D> Queue<D>
where
  D: Copy,
{
  #[inline]
  pub(crate) fn push_front(&mut self, value: D) -> Result<(), QueueError> {
    self.reserve(1).map_err(|_err| QueueError::PushFrontOverflow)?;
    let len = self.data.len();
    self.head = wrap_sub(self.data.capacity(), self.head, 1);
    // SAFETY: `self.head` points to valid memory
    let dst = unsafe { self.data.as_mut_ptr().add(self.head) };
    // SAFETY: `dst` points to valid memory
    unsafe {
      ptr::write(dst, value);
    }
    // SAFETY: top-level check ensures capacity
    let new_len = unsafe { len.unchecked_add(1) };
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    Ok(())
  }

  #[inline(always)]
  pub(crate) fn reserve(&mut self, additional: usize) -> Result<(), QueueError> {
    reserve(additional, &mut self.data, &mut self.head)
      .map(|_el| ())
      .map_err(|_err| QueueError::ReserveOverflow)
  }
}

impl<D> Debug for Queue<D>
where
  D: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    let (lhs, rhs) = self.as_slices();
    f.debug_struct("Queue").field("lhs", &lhs).field("rhs", &rhs).finish()
  }
}

impl<D> Default for Queue<D> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "_proptest")]
#[cfg(test)]
mod _proptest {
  use crate::misc::Queue;
  use alloc::{collections::VecDeque, vec::Vec};

  #[test_strategy::proptest]
  fn queue(bytes: Vec<u8>) {
    let mut queue = Queue::with_capacity(bytes.len()).unwrap();
    let mut vec_deque = VecDeque::with_capacity(bytes.len());

    for byte in bytes.iter().copied() {
      queue.push_front(byte).unwrap();
      vec_deque.push_front(byte);
    }
    assert_eq!((queue.capacity(), queue.len()), (vec_deque.capacity(), vec_deque.len()));
    for _ in 0..(bytes.len() / 2) {
      assert_eq!(queue.as_slices(), vec_deque.as_slices());
      assert_eq!(queue.get(0), vec_deque.get(0));
      assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
      assert_eq!(queue.pop_back(), vec_deque.pop_back());
      assert_eq!(queue.as_slices(), vec_deque.as_slices());
      assert_eq!(queue.get(0), vec_deque.get(0));
      assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
      assert_eq!(queue.pop_front(), vec_deque.pop_front());
    }
    queue.reserve(queue.capacity() + 10).unwrap();
    vec_deque.reserve(vec_deque.capacity() + 10);
    loop {
      if queue.len() == 0 {
        break;
      }
      assert_eq!(queue.as_slices(), vec_deque.as_slices());
      assert_eq!(queue.get(0), vec_deque.get(0));
      assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
      assert_eq!(queue.pop_back(), vec_deque.pop_back());
      if queue.len() == 0 {
        break;
      }
      assert_eq!(queue.as_slices(), vec_deque.as_slices());
      assert_eq!(queue.get(0), vec_deque.get(0));
      assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
      assert_eq!(queue.pop_front(), vec_deque.pop_front());
    }
    assert_eq!((queue.capacity(), queue.len()), (vec_deque.capacity(), vec_deque.len()));
    assert_eq!((queue.len(), vec_deque.len()), (0, 0));
  }
}

#[cfg(test)]
mod tests {
  use crate::misc::Queue;

  #[test]
  fn as_slices() {
    let mut queue = Queue::with_capacity(4).unwrap();
    queue.push_front(1).unwrap();
    queue.push_front(2).unwrap();
    queue.push_front(3).unwrap();
    queue.push_front(4).unwrap();
    let _ = queue.pop_back();
    let _ = queue.pop_back();
    queue.push_front(5).unwrap();
    assert_eq!(queue.as_slices(), (&[4, 3][..], &[5][..]));
  }

  #[test]
  fn clear() {
    let mut queue = Queue::with_capacity(1).unwrap();
    assert_eq!(queue.len(), 0);
    queue.push_front(1).unwrap();
    assert_eq!(queue.len(), 1);
    queue.clear();
    assert_eq!(queue.len(), 0);
  }

  #[test]
  fn get() {
    let mut queue = Queue::with_capacity(1).unwrap();
    assert_eq!(queue.get(0), None);
    assert_eq!(queue.get_mut(0), None);
    queue.push_front(1).unwrap();
    assert_eq!(queue.get(0), Some(&1i32));
    assert_eq!(queue.get_mut(0), Some(&mut 1i32));
  }

  #[test]
  fn pop_back() {
    let mut queue = Queue::with_capacity(1).unwrap();
    assert_eq!(queue.pop_back(), None);
    queue.push_front(1).unwrap();
    assert_eq!(queue.pop_back(), Some(1));
    assert_eq!(queue.pop_back(), None);
  }

  #[test]
  fn pop_front() {
    let mut queue = Queue::with_capacity(1).unwrap();
    assert_eq!(queue.pop_front(), None);
    queue.push_front(1).unwrap();
    assert_eq!(queue.pop_front(), Some(1));
    assert_eq!(queue.pop_front(), None);
  }

  #[test]
  fn push_front() {
    let mut queue = Queue::with_capacity(1).unwrap();
    assert_eq!(queue.len(), 0);
    queue.push_front(1).unwrap();
    assert_eq!(queue.len(), 1);
  }

  #[test]
  fn push_when_full() {
    let mut bq = Queue::with_capacity(5).unwrap();
    bq.push_front(0).unwrap();
    bq.push_front(1).unwrap();
    bq.push_front(2).unwrap();
    bq.push_front(3).unwrap();
    bq.push_front(4).unwrap();
    let _ = bq.pop_back();
    let _ = bq.pop_back();
    bq.push_front(5).unwrap();
    bq.push_front(6).unwrap();
    assert_eq!(bq.as_slices(), (&[4, 3, 2][..], &[6, 5][..]));
  }

  #[test]
  fn reserve() {
    let mut queue = Queue::<u8>::new();
    assert_eq!(queue.capacity(), 0);
    queue.reserve(10).unwrap();
    assert_eq!(queue.capacity(), 10);
  }
}
