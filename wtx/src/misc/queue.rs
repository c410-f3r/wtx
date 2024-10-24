macro_rules! as_slices {
  ($empty:expr, $ptr:ident, $slice:ident, $this:expr, $($ref:tt)*) => {{
    let capacity = $this.data.capacity();
    let head = $this.head;
    let len = $this.data.len();
    let ptr = $this.data.$ptr();
    let tail = $this.tail;
    // SAFETY: inner data is expected to point to valid memory
    let head_ptr = unsafe { ptr.add(head) };
    if is_wrapping(head, len, tail) {
      let front_len = capacity.wrapping_sub(head);
      // SAFETY: `ìf` check ensures bounds
      let front = unsafe { $($ref)* *ptr::$slice(head_ptr, front_len) };
      // SAFETY: `ìf` check ensures bounds
      let back = unsafe { $($ref)* *ptr::$slice(ptr, tail) };
      (front, back)
    } else {
      // SAFETY: inner data is expected to point to valid memory
      let front = unsafe { $($ref)* *ptr::$slice(head_ptr, len) };
      (front, $empty)
    }
  }}
}

#[cfg(all(feature = "_proptest", test))]
mod proptest;
#[cfg(test)]
mod tests;

use crate::misc::Vector;
use core::{
  fmt::{Debug, Formatter},
  mem::needs_drop,
  ptr, slice,
};

/// Errors of [Queue].
#[derive(Debug)]
pub enum QueueError {
  #[doc = doc_single_elem_cap_overflow!()]
  ExtendFromSliceOverflow,
  #[doc = doc_single_elem_cap_overflow!()]
  PushFrontOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
  #[doc = doc_reserve_overflow!()]
  WithCapacityOverflow,
}

/// A circular buffer.
pub struct Queue<T> {
  data: Vector<T>,
  head: usize,
  tail: usize,
}

impl<T> Queue<T> {
  const NEEDS_DROP: bool = needs_drop::<T>();

  /// Creates a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Vector::new(), head: 0, tail: 0 }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(cap: usize) -> Result<Self, QueueError> {
    Ok(Self {
      data: Vector::with_capacity(cap).map_err(|_err| QueueError::WithCapacityOverflow)?,
      head: 0,
      tail: 0,
    })
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_exact_capacity(cap: usize) -> Result<Self, QueueError> {
    Ok(Self {
      data: Vector::with_capacity(cap).map_err(|_err| QueueError::WithCapacityOverflow)?,
      head: 0,
      tail: 0,
    })
  }

  /// Returns a raw pointer to the vector's buffer, or a dangling raw pointer
  /// valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.data.as_ptr()
  }

  /// Returns an unsafe mutable pointer to the queue's buffer, or a dangling
  /// raw pointer valid for zero sized reads if the vector didn't allocate.
  #[inline]
  pub fn as_ptr_mut(&mut self) -> *mut T {
    self.data.as_mut_ptr()
  }

  /// Returns a pair of slices which contain, in order, the contents of the queue.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::with_capacity(8).unwrap();
  /// queue.push_front(3).unwrap();
  /// queue.push_back(1).unwrap();
  /// queue.push_back(2).unwrap();
  /// assert_eq!(queue.as_slices(), (&[3][..], &[1, 2][..]));
  /// ```
  #[inline]
  pub fn as_slices(&self) -> (&[T], &[T]) {
    as_slices!(&[][..], as_ptr, slice_from_raw_parts, self, &)
  }

  /// Mutable version of [`Self::as_slices`].
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_front(3);
  /// queue.push_back(1);
  /// queue.push_back(2);
  /// assert_eq!(queue.as_slices_mut(), (&mut [3][..], &mut [1, 2][..]));
  /// ```
  #[inline]
  pub fn as_slices_mut(&mut self) -> (&mut [T], &mut [T]) {
    as_slices!(&mut [][..], as_ptr_mut, slice_from_raw_parts_mut, self, &mut)
  }

  /// Returns the number of elements the queue can hold without reallocating.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.data.capacity()
  }

  /// Clears the queue, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    let Self { data, head, tail } = self;
    data.clear();
    *head = 0;
    *tail = 0;
  }

  /// Provides a reference to the element at the given index.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// assert_eq!(queue.get(0), Some(&1));
  /// ```
  #[inline]
  pub fn get(&self, mut idx: usize) -> Option<&T> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` points to valid memory
    let rslt = unsafe { self.data.as_ptr().add(idx) };
    // SAFETY: `idx` points to valid memory
    unsafe { Some(&*rslt) }
  }

  /// Mutable version of [`Self::get`].
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// assert_eq!(queue.get_mut(0), Some(&mut 1));
  /// ```
  #[inline]
  pub fn get_mut(&mut self, mut idx: usize) -> Option<&mut T> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` points to valid memory
    let rslt = unsafe { self.data.as_ptr_mut().add(idx) };
    // SAFETY: `idx` points to valid memory
    unsafe { Some(&mut *rslt) }
  }

  /// Returns a front-to-back iterator.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_front(3);
  /// let mut iter = queue.iter();
  /// assert_eq!(iter.next(), Some(&3));
  /// assert_eq!(iter.next(), Some(&1));
  /// assert_eq!(iter.next(), None);
  /// ```
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = &T> {
    let (front, back) = self.as_slices();
    front.iter().chain(back)
  }

  /// Mutable version of [`Self::iter`].
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_front(3);
  /// let mut iter = queue.iter_mut();
  /// assert_eq!(iter.next(), Some(&mut 3));
  /// assert_eq!(iter.next(), Some(&mut 1));
  /// assert_eq!(iter.next(), None);
  /// ```
  #[inline]
  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
    let (front, back) = self.as_slices_mut();
    front.iter_mut().chain(back)
  }

  /// Returns the last element.
  #[inline]
  pub fn last(&self) -> Option<&T> {
    self.get(self.len().checked_sub(1)?)
  }

  /// Returns the number of elements.
  #[inline]
  pub fn len(&self) -> usize {
    self.data.len()
  }

  /// Removes the last element from the queue and returns it, or `None` if it is empty.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// queue.pop_back();
  /// assert_eq!(queue.as_slices(), (&[1][..], &[][..]));
  /// ```
  #[inline]
  pub fn pop_back(&mut self) -> Option<T> {
    let new_len = self.data.len().checked_sub(1)?;
    let curr_tail = wrap_sub(self.data.capacity(), self.tail, 1);
    self.tail = curr_tail;
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    // SAFETY: `idx` points to valid memory
    let src = unsafe { self.data.as_ptr_mut().add(curr_tail) };
    // SAFETY: `src` points to valid memory
    Some(unsafe { ptr::read(src) })
  }

  /// Removes the first element and returns it, or [`Option::None`] if the queue is empty.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// queue.pop_front();
  /// assert_eq!(queue.as_slices(), (&[3][..], &[][..]));
  /// ```
  #[inline]
  pub fn pop_front(&mut self) -> Option<T> {
    let new_len = self.data.len().checked_sub(1)?;
    let prev_head = self.head;
    self.head = wrap_add(self.data.capacity(), prev_head, 1);
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    // SAFETY: `prev_head` points to valid memory
    let src = unsafe { self.data.as_ptr_mut().add(prev_head) };
    // SAFETY: `src` points to valid memory
    Some(unsafe { ptr::read(src) })
  }

  /// Appends an element to the back of the queue.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// assert_eq!(queue.as_slices(), (&[1, 3][..], &[][..]));
  /// ```
  #[inline]
  pub fn push_back(&mut self, value: T) -> Result<(), QueueError> {
    let _ = self.reserve_back(1).map_err(|_err| QueueError::PushFrontOverflow)?;
    let len = self.data.len();
    let tail = self.tail;
    self.tail = wrap_add(self.data.capacity(), tail, 1);
    // SAFETY: `idx` is within bounds
    let dst = unsafe { self.data.as_ptr_mut().add(tail) };
    // SAFETY: `dst` points to valid memory
    unsafe {
      ptr::write(dst, value);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(len.wrapping_add(1));
    }
    Ok(())
  }

  /// Prepends an element to the queue.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_front(1);
  /// queue.push_front(3);
  /// assert_eq!(queue.as_slices(), (&[3, 1][..], &[][..]));
  /// ```
  #[inline]
  pub fn push_front(&mut self, value: T) -> Result<(), QueueError> {
    let _ = self.reserve_front(1).map_err(|_err| QueueError::PushFrontOverflow)?;
    let len = self.data.len();
    self.head = wrap_sub(self.data.capacity(), self.head, 1);
    // SAFETY: `self.head` points to valid memory
    let dst = unsafe { self.data.as_ptr_mut().add(self.head) };
    // SAFETY: `dst` points to valid memory
    unsafe {
      ptr::write(dst, value);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(len.wrapping_add(1));
    }
    Ok(())
  }

  /// Reserves capacity for at least additional more elements to be inserted at the back of the
  /// queue.
  #[inline(always)]
  pub fn reserve_back(&mut self, additional: usize) -> Result<usize, QueueError> {
    let tuple = reserve::<_, true>(additional, &mut self.data, &mut self.head, &mut self.tail)?;
    Ok(tuple.2)
  }

  /// Reserves capacity for at least additional more elements to be inserted at the front of the
  /// queue.
  #[inline(always)]
  pub fn reserve_front(&mut self, additional: usize) -> Result<usize, QueueError> {
    let tuple = reserve::<_, false>(additional, &mut self.data, &mut self.head, &mut self.tail)?;
    Ok(tuple.2)
  }

  /// Shortens the queue, keeping the first `new_len` elements.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_front(1);
  /// queue.push_front(3);
  /// queue.push_back(5);
  /// queue.push_back(7);
  /// queue.truncate_back(2);
  /// assert_eq!(queue.as_slices(), (&[3, 1][..], &[][..]));
  /// queue.truncate_back(0);
  /// assert_eq!(queue.as_slices(), (&[][..], &[][..]));
  /// ```
  #[inline]
  pub fn truncate_back(&mut self, new_len: usize) {
    let len = self.data.len();
    let Some(diff @ 1..=usize::MAX) = len.checked_sub(new_len) else {
      return;
    };
    if is_wrapping(self.head, len, self.tail) {
      if let Some(back_begin) = self.tail.checked_sub(diff) {
        if Self::NEEDS_DROP {
          // SAFETY: Indices are within bounds
          unsafe {
            drop_elements(diff, back_begin, self.data.as_ptr_mut());
          }
        }
        self.tail = if self.tail == diff { self.data.capacity() } else { back_begin }
      } else {
        let front_len = diff.wrapping_sub(self.tail);
        let front_begin = self.data.capacity().wrapping_sub(front_len);
        if Self::NEEDS_DROP {
          // SAFETY: Indices are within bounds
          unsafe {
            drop_elements(self.tail, 0, self.data.as_ptr_mut());
          }
          // SAFETY: Indices are within bounds
          unsafe {
            drop_elements(front_len, front_begin, self.data.as_ptr_mut());
          }
        }
        self.tail = front_begin;
      }
    } else {
      let curr_tail = self.tail.wrapping_sub(diff);
      if Self::NEEDS_DROP {
        // SAFETY: Indices are within bounds
        unsafe {
          drop_elements(diff, curr_tail, self.data.as_ptr_mut());
        }
      }
      self.tail = curr_tail;
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
  }

  /// Shortens the queue, keeping the last `new_len` elements.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_front(1);
  /// queue.push_front(3);
  /// queue.push_back(5);
  /// queue.push_back(7);
  /// queue.truncate_front(2);
  /// assert_eq!(queue.as_slices(), (&[5, 7][..], &[][..]));
  /// queue.truncate_front(0);
  /// assert_eq!(queue.as_slices(), (&[][..], &[][..]));
  /// ```
  #[inline]
  pub fn truncate_front(&mut self, new_len: usize) {
    let len = self.data.len();
    let Some(diff @ 1..=usize::MAX) = len.checked_sub(new_len) else {
      return;
    };
    if is_wrapping(self.head, len, self.tail) {
      let front_slots = self.data.capacity().wrapping_sub(self.head);
      if front_slots >= diff {
        if Self::NEEDS_DROP {
          // SAFETY: Indices are within bounds
          unsafe {
            drop_elements(diff, self.head, self.data.as_ptr_mut());
          }
        }
        self.head = if front_slots == diff { 0 } else { self.head.wrapping_add(diff) }
      } else {
        let back_len = diff.wrapping_sub(front_slots);
        if Self::NEEDS_DROP {
          // SAFETY: Indices are within bounds
          unsafe {
            drop_elements(front_slots, self.head, self.data.as_ptr_mut());
          }
          // SAFETY: Indices are within bounds
          unsafe {
            drop_elements(back_len, 0, self.data.as_ptr_mut());
          }
        }
        self.head = back_len;
      }
    } else {
      let prev_head = self.head;
      if Self::NEEDS_DROP {
        // SAFETY: Indices are within bounds
        unsafe {
          drop_elements(diff, prev_head, self.data.as_ptr_mut());
        }
      }
      self.head = prev_head.wrapping_add(diff);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
  }
}

impl<T> Queue<T>
where
  T: Copy,
{
  /// Iterates over the `others` slices, copies each element, and then prepends
  /// it to this vector. The `others` slices are traversed in-order.
  ///
  /// ```rust
  /// let mut queue = wtx::misc::Queue::new();
  /// queue.push_front(4);
  /// queue.extend_front_from_copyable_slices([&[2, 3][..]]);
  /// queue.extend_front_from_copyable_slices([&[0, 1][..], &[1][..]]);
  /// assert_eq!(queue.as_slices(), (&[0, 1, 1, 2, 3, 4][..], &[][..]));
  /// ```
  #[inline]
  pub fn extend_front_from_copyable_slices<'iter, I>(
    &mut self,
    others: I,
  ) -> Result<(usize, usize), QueueError>
  where
    I: IntoIterator<Item = &'iter [T]>,
    I::IntoIter: Clone,
    T: 'iter,
  {
    let mut others_len: usize = 0;
    let iter = others.into_iter();
    for other in iter.clone() {
      let Some(curr_len) = others_len.checked_add(other.len()) else {
        return Err(QueueError::ExtendFromSliceOverflow);
      };
      others_len = curr_len;
    }
    let tuple = reserve::<_, false>(others_len, &mut self.data, &mut self.head, &mut self.tail)?;
    let mut head = tuple.0;
    self.head = head;
    for other in iter {
      // SAFETY: `self.head` points to valid memory
      let dst = unsafe { self.data.as_ptr_mut().add(head) };
      // SAFETY: `dst` points to valid memory
      unsafe {
        ptr::copy_nonoverlapping(other.as_ptr(), dst, other.len());
      }
      head = head.wrapping_add(other.len());
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(self.data.len().wrapping_add(others_len));
    }
    Ok((others_len, tuple.2))
  }

  pub(crate) fn head(&self) -> usize {
    self.head
  }
}

impl<T> Debug for Queue<T>
where
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    let (front, back) = self.as_slices();
    f.debug_struct("Queue").field("front", &front).field("back", &back).finish()
  }
}

impl<T> Default for Queue<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[inline]
unsafe fn drop_elements<T>(len: usize, offset: usize, ptr: *mut T) {
  // SAFETY: It is up to the caller to provide a valid pointer with a valid index
  let data = unsafe { ptr.add(offset) };
  // SAFETY: It is up to the caller to provide a valid length
  let elements = unsafe { slice::from_raw_parts_mut(data, len) };
  // SAFETY: It is up to the caller to provide parameters that can lead to droppable elements
  unsafe {
    ptr::drop_in_place(elements);
  }
}

/// All wrapping structures should have at least 1 element.
///
/// ```txt
/// H = Head (inclusive)
/// T = Tail (exclusive)
///
/// H(7) > T(0): . . . . . . . H (wrapping)
/// H(7) > T(1): T . . . . . . H (wrapping)
/// H(6) > T(4): * * * T . . H * (wrapping)
///
/// H(0) = T(0): . . . . . . . . (no wrapping)
/// H(1) = T(1): . . . . . . . . (no wrapping)
/// H(0) = T(0): H * * * * * * T (wrapping)
/// H(2) = T(2): * T H * * * * * (wrapping)
/// H(7) = T(7): * * * * * * T H (wrapping)
///
/// H(0) < T(1): H . . . . . . . (no wrapping)
/// H(7) < T(8): . . . . . . . H (no wrapping)
/// H(0) < T(2): H T . . . . . . (no wrapping)
/// H(3) < T(6): . . . H * T . . (no wrapping)
/// H(0) < T(8): H * * * * * * T (no wrapping)
/// ```
#[inline]
fn is_wrapping(head: usize, len: usize, tail: usize) -> bool {
  if tail > head {
    false
  } else {
    len > 0
  }
}

/// Returns the starting and ending index where the `additional` number of elements
/// can be inserted.
#[inline(always)]
fn reserve<D, const IS_BACK: bool>(
  additional: usize,
  data: &mut Vector<D>,
  head: &mut usize,
  tail: &mut usize,
) -> Result<(usize, usize, usize), QueueError> {
  let len = data.len();
  let prev_cap = data.capacity();
  data.reserve(additional).map_err(|_err| QueueError::ReserveOverflow)?;
  let curr_cap = data.capacity();
  let prev_head = prev_cap.min(*head);
  let prev_tail = prev_cap.min(*tail);
  if len == 0 {
    return Ok(if IS_BACK {
      (0, additional, 0)
    } else {
      (curr_cap.wrapping_sub(additional), curr_cap, 0)
    });
  }
  if is_wrapping(prev_head, len, prev_tail) {
    let free_slots = prev_head.wrapping_sub(prev_tail);
    if free_slots >= additional {
      return Ok(if IS_BACK {
        (prev_tail, prev_tail.wrapping_add(additional), 0)
      } else {
        (prev_head.wrapping_sub(additional), prev_head, 0)
      });
    }
    let front_len = prev_cap.wrapping_sub(prev_head);
    let curr_head = curr_cap.wrapping_sub(front_len);
    // SAFETY: `prev_head` is equal or less than the current capacity
    let src = unsafe { data.as_ptr_mut().add(prev_head) };
    // SAFETY: `curr_head` is equal or less than the current capacity
    let dst = unsafe { data.as_ptr_mut().add(curr_head) };
    // SAFETY: memory has been allocated
    unsafe {
      ptr::copy(src, dst, front_len);
    }
    *head = curr_head;
    if IS_BACK {
      Ok((prev_tail, prev_tail.wrapping_add(additional), 0))
    } else {
      Ok((curr_head.wrapping_sub(additional), curr_head, curr_cap.wrapping_sub(prev_cap)))
    }
  } else {
    let left_free = prev_head;
    let right_free = curr_cap.wrapping_sub(prev_tail);
    if IS_BACK {
      if right_free >= additional {
        return Ok((prev_tail, prev_tail.wrapping_add(additional), 0));
      }
      if right_free == 0 && left_free >= additional {
        return Ok((0, additional, 0));
      }
      // SAFETY: `prev_head` is equal or less than the current capacity
      let src = unsafe { data.as_ptr_mut().add(prev_head) };
      // SAFETY: memory has been allocated
      unsafe {
        ptr::copy(src, data.as_ptr_mut(), len);
      }
      let curr_tail = len;
      *head = 0;
      *tail = curr_tail;
      Ok((curr_tail, curr_tail.wrapping_add(additional), 0))
    } else {
      if left_free >= additional {
        return Ok((prev_head.wrapping_sub(additional), prev_head, 0));
      }
      if left_free == 0 && right_free >= additional {
        return Ok((curr_cap.wrapping_sub(additional), curr_cap, 0));
      }
      let curr_head = curr_cap.wrapping_sub(len);
      // SAFETY: `prev_head` is equal or less than the current capacity
      let src = unsafe { data.as_ptr_mut().add(prev_head) };
      // SAFETY: `curr_head` is equal or less than the current capacity
      let dst = unsafe { data.as_ptr_mut().add(curr_head) };
      // SAFETY: memory has been allocated
      unsafe {
        ptr::copy(src, dst, len);
      }
      *head = curr_head;
      *tail = curr_cap;
      Ok((curr_head.wrapping_sub(additional), curr_head, right_free))
    }
  }
}

#[inline]
fn wrap_add(capacity: usize, idx: usize, value: usize) -> usize {
  wrap_idx(idx.wrapping_add(value), capacity)
}

#[inline]
fn wrap_idx(idx: usize, cap: usize) -> usize {
  idx.checked_sub(cap).unwrap_or(idx)
}

#[inline]
fn wrap_sub(capacity: usize, idx: usize, value: usize) -> usize {
  #[inline]
  fn wrap_idx(idx: usize, cap: usize) -> usize {
    idx.checked_sub(cap).unwrap_or(idx)
  }
  wrap_idx(idx.wrapping_sub(value).wrapping_add(capacity), capacity)
}
