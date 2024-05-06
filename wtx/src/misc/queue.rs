macro_rules! as_slices {
  ($empty:expr, $ptr:ident, $slice:ident, $this:expr, $($ref:tt)*) => {{
    let len = $this.data.len();
    let rhs_len = $this.capacity().wrapping_sub($this.head);
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

use crate::misc::{
  queue_utils::{reserve, wrap_add, wrap_sub},
  Vector,
};
use core::{
  fmt::{Debug, Formatter},
  ptr,
};

/// A circular buffer where elements are added in only one-way.
pub struct Queue<D> {
  data: Vector<D>,
  head: usize,
}

impl<D> Queue<D>
where
  D: Copy,
{
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { data: Vector::new(), head: 0 }
  }

  #[inline]
  pub(crate) fn with_capacity(cap: usize) -> Self {
    Self { data: Vector::with_capacity(cap), head: 0 }
  }

  #[inline]
  pub(crate) fn as_slices(&self) -> (&[D], &[D]) {
    as_slices!(&[][..], as_ptr, slice_from_raw_parts, self, &)
  }

  #[inline]
  pub(crate) fn as_slices_mut(&mut self) -> (&mut [D], &mut [D]) {
    as_slices!(&mut [][..], as_mut_ptr, slice_from_raw_parts_mut, self, &mut)
  }

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
    // SAFETY: `idx` is less than the current length
    unsafe { Some(&*self.data.as_ptr().add(idx)) }
  }

  #[inline]
  pub(crate) fn get_mut(&mut self, mut idx: usize) -> Option<&mut D> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` is less than the current length
    unsafe { Some(&mut *self.data.as_mut_ptr().add(idx)) }
  }

  #[inline]
  pub(crate) fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  #[inline]
  pub(crate) fn is_full(&self) -> bool {
    self.data.len() >= self.data.capacity()
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
    if self.is_empty() {
      return None;
    }
    // SAFETY: Structure is not empty
    unsafe {
      self.data.set_len(self.data.len().unchecked_sub(1));
      let idx = wrap_add(self.data.capacity(), self.head, self.data.len());
      Some(ptr::read(self.data.as_mut_ptr().add(idx)))
    }
  }

  #[inline]
  pub(crate) fn pop_front(&mut self) -> Option<D> {
    if self.is_empty() {
      return None;
    }
    let len = self.data.len();
    let prev_head = self.head;
    self.head = wrap_add(self.data.capacity(), self.head, 1);
    // SAFETY: Structure is not empty
    unsafe {
      self.data.set_len(len.unchecked_sub(1));
      Some(ptr::read(self.data.as_mut_ptr().add(prev_head)))
    }
  }

  #[inline]
  pub(crate) fn push_front(&mut self, element: D) -> crate::Result<()> {
    if self.is_full() {
      return Err(crate::Error::CapacityOverflow);
    }
    let len = self.data.len();
    self.head = wrap_sub(self.data.capacity(), self.head, 1);
    // SAFETY: There is enough capacity
    unsafe {
      ptr::write(self.data.as_mut_ptr().add(self.head), element);
      self.data.set_len(len.unchecked_add(1));
    }
    Ok(())
  }

  #[inline(always)]
  pub(crate) fn reserve(&mut self, additional: usize) {
    let _ = reserve(additional, &mut self.data, &mut self.head);
  }
}

impl<D> Debug for Queue<D>
where
  D: Copy + Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    let (lhs, rhs) = self.as_slices();
    f.debug_struct("Queue").field("lhs", &lhs).field("rhs", &rhs).finish()
  }
}

impl<D> Default for Queue<D>
where
  D: Copy,
{
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
    let mut queue = Queue::with_capacity(bytes.len());
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
    queue.reserve(queue.capacity() + 10);
    vec_deque.reserve(vec_deque.capacity() + 10);
    loop {
      if queue.is_empty() {
        break;
      }
      assert_eq!(queue.as_slices(), vec_deque.as_slices());
      assert_eq!(queue.get(0), vec_deque.get(0));
      assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
      assert_eq!(queue.pop_back(), vec_deque.pop_back());
      if queue.is_empty() {
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
  fn clear() {
    let mut queue = Queue::with_capacity(1);
    assert_eq!(queue.len(), 0);
    queue.push_front(1).unwrap();
    assert_eq!(queue.len(), 1);
    queue.clear();
    assert_eq!(queue.len(), 0);
  }

  #[test]
  fn get() {
    let mut queue = Queue::with_capacity(1);
    assert_eq!(queue.get(0), None);
    assert_eq!(queue.get_mut(0), None);
    queue.push_front(1).unwrap();
    assert_eq!(queue.get(0), Some(&1i32));
    assert_eq!(queue.get_mut(0), Some(&mut 1i32));
  }

  #[test]
  fn pop_back() {
    let mut queue = Queue::with_capacity(1);
    assert_eq!(queue.pop_back(), None);
    queue.push_front(1).unwrap();
    assert_eq!(queue.pop_back(), Some(1));
    assert_eq!(queue.pop_back(), None);
  }

  #[test]
  fn pop_front() {
    let mut queue = Queue::with_capacity(1);
    assert_eq!(queue.pop_front(), None);
    queue.push_front(1).unwrap();
    assert_eq!(queue.pop_front(), Some(1));
    assert_eq!(queue.pop_front(), None);
  }

  #[test]
  fn push_front() {
    let mut queue = Queue::with_capacity(1);
    assert_eq!(queue.len(), 0);
    queue.push_front(1).unwrap();
    assert_eq!(queue.len(), 1);
  }

  #[test]
  fn reserve() {
    let mut queue = Queue::<u8>::new();
    assert_eq!(queue.capacity(), 0);
    queue.reserve(10);
    assert_eq!(queue.capacity(), 10);
  }
}
