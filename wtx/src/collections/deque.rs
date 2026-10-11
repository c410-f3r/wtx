// 1. Valid instances
//
// In a double ended queue it is possible to store elements in 2 logical ways.
//
// 1.1. Contiguous
//
// No boundary intersections
//
// |   |   | A | B | C |   |   |   |   |
//
// 1.2. Wrapping
//
// The order doesn't matter, front elements will always stay at the right-hand-side
// and back elements will always stay at the left-hand-side.
//
// 1.2.1 Pushing an element to the back of the queue.
//
// |   |   |   |   |   |   |   | A | B |
// -------------------------------------
// | C |   |   |   |   |   |   | A | B |
//
// 1.2.2 Prepending an element to the front of the queue.
//
// | A | B |   |   |   |   |   |   |   |
// -------------------------------------
// | B | C |   |   |   |   |   |   | A |
//
// 2. Invalid instances
//
// It is impossible to exist a wrapping non-contiguous queue like in the following examples.
//
// |   | A | B |   | C |   |   |   |   |
//
// | B | C | D |   |   |   | A |   |   |

macro_rules! as_slices {
  ($empty:expr, $ptr:ident, $slice:ident, $this:expr, $($ref:tt)*) => {{
    let capacity = $this.data.capacity();
    let head = $this.head;
    let len = $this.data.len();
    let ptr = $this.data.$ptr();
    let tail = $this.tail;
    // SAFETY: inner data is expected to point to valid memory
    let head_ptr = unsafe { ptr.add(head.usize()) };
    if is_wrapping(head, len, tail) {
      let front_len = capacity.wrapping_sub(head);
      // SAFETY: `ìf` check ensures bounds
      let front = unsafe { $($ref)* *ptr::$slice(head_ptr, front_len.usize()) };
      // SAFETY: `ìf` check ensures bounds
      let back = unsafe { $($ref)* *ptr::$slice(ptr, tail.usize()) };
      (front, back)
    } else {
      // SAFETY: inner data is expected to point to valid memory
      let front = unsafe { $($ref)* *ptr::$slice(head_ptr, len.usize()) };
      (front, $empty)
    }
  }}
}

#[cfg(kani)]
mod kani;
#[cfg(test)]
mod tests;

use crate::collections::{ExpansionTy, LinearStorageLen, TryExtend, Vector, misc::drop_elements};
use core::{
  fmt::{Debug, Formatter},
  mem::needs_drop,
  ptr, slice,
};

/// [`Deque`] with a capacity limited by `u8`.
pub type DequeU8<T> = Deque<u8, T>;
/// [`Deque`] with a capacity limited by `u16`.
pub type DequeU16<T> = Deque<u16, T>;
/// [`Deque`] with a capacity limited by `u32`.
pub type DequeU32<T> = Deque<u32, T>;
/// [`Deque`] with a capacity limited by `usize`.
pub type DequeUsize<T> = Deque<usize, T>;

/// Errors of [Deque].
#[derive(Clone, Copy, Debug)]
pub enum DequeueError {
  #[doc = doc_single_elem_cap_overflow!()]
  ExtendFromSliceOverflow,
  /// The provided range does not point to valid internal data
  OutOfBoundsRange,
  #[doc = doc_single_elem_cap_overflow!()]
  PushFrontOverflow,
  #[doc = doc_reserve_overflow!()]
  ReserveOverflow,
  #[doc = doc_reserve_overflow!()]
  WithCapacityOverflow,
}

/// A double-ended queue implemented with a growable ring buffer.
//
// # Illustration
//
// |   |   | A | B | C | D |   |   |   |   |
//         |               |               |--> data.capacity()
//         |               |
//         |               |------------------> tail
//         |
//         |----------------------------------> head
//
// The vector length is a shortcut for the sum of head and tail elements.
pub struct Deque<L, T>
where
  L: LinearStorageLen,
{
  data: Vector<L, T>,
  head: L,
  tail: L,
}

impl<L, T> Deque<L, T>
where
  L: LinearStorageLen,
{
  const NEEDS_DROP: bool = needs_drop::<T>();

  /// Creates a new empty instance.
  #[inline]
  pub const fn new() -> Self {
    Self { data: Vector::new(), head: L::ZERO, tail: L::ZERO }
  }

  /// Constructs a new, empty instance with at least the specified capacity.
  #[inline]
  pub fn with_capacity(capacity: L) -> crate::Result<Self> {
    Ok(Self {
      data: Vector::with_capacity(capacity).map_err(|_err| DequeueError::WithCapacityOverflow)?,
      head: L::ZERO,
      tail: L::ZERO,
    })
  }

  /// Constructs a new, empty instance with the specified capacity.
  #[inline]
  pub fn with_exact_capacity(capacity: L) -> crate::Result<Self> {
    Ok(Self {
      data: Vector::with_capacity(capacity).map_err(|_err| DequeueError::WithCapacityOverflow)?,
      head: L::ZERO,
      tail: L::ZERO,
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
  /// let mut queue = wtx::collections::DequeU8::with_capacity(8).unwrap();
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
  /// let mut queue = wtx::collections::DequeU8::new();
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
  pub fn capacity(&self) -> L {
    self.data.capacity()
  }

  /// Clears the queue, removing all values.
  #[inline]
  pub fn clear(&mut self) {
    let Self { data, head, tail } = self;
    data.clear();
    *head = L::ZERO;
    *tail = L::ZERO;
  }

  /// Appends elements to the back of the instance so that the current length is equal to `et`.
  ///
  /// Does nothing if the calculated length is equal or less than the current length.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.expand_back(wtx::collections::ExpansionTy::Len(4), 1);
  /// assert_eq!(queue.as_slices(), (&[1, 1, 1, 1][..], &[][..]));
  /// ```
  #[inline(always)]
  pub fn expand_back(&mut self, et: ExpansionTy<L>, value: T) -> crate::Result<L>
  where
    T: Clone,
  {
    let len = self.data.len();
    let Some((additional, new_len)) = et.params(len) else {
      return Ok(L::ZERO);
    };
    let rr = self.prolong_back(additional)?;
    // SAFETY: elements were allocated
    unsafe {
      self.expand(additional, rr.begin, new_len, value);
    }
    self.tail = rr.begin.wrapping_add(additional);
    Ok(additional)
  }

  /// Prepends elements to the front of the instance so that the current length is equal to `et`.
  ///
  /// Does nothing if the calculated length is equal or less than the current length.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.expand_front(wtx::collections::ExpansionTy::Len(4), 1);
  /// assert_eq!(queue.as_slices(), (&[1, 1, 1, 1][..], &[][..]));
  /// ```
  #[inline(always)]
  pub fn expand_front(&mut self, et: ExpansionTy<L>, value: T) -> crate::Result<(L, L)>
  where
    T: Clone,
  {
    let len = self.data.len();
    let Some((additional, new_len)) = et.params(len) else {
      return Ok((L::ZERO, L::ZERO));
    };
    let rr = self.prolong_front(additional)?;
    // SAFETY: elements were allocated
    unsafe {
      self.expand(additional, rr.begin, new_len, value);
    }
    self.head = rr.begin;
    Ok((additional, rr.head_shift))
  }

  /// Appends all elements of the iterator.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.extend_back_from_iter([1, 2]);
  /// assert_eq!(queue.len(), 2);
  /// ```
  #[inline]
  pub fn extend_back_from_iter(&mut self, ii: impl IntoIterator<Item = T>) -> crate::Result<()> {
    let iter = ii.into_iter();
    let _ = self.reserve_back(L::from_usize(iter.size_hint().0)?)?;
    for elem in iter {
      self.push_back(elem)?;
    }
    Ok(())
  }

  /// Prepends all elements of the iterator.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.extend_front_from_iter([1, 2]);
  /// assert_eq!(queue.len(), 2);
  /// ```
  #[inline]
  pub fn extend_front_from_iter(&mut self, ii: impl IntoIterator<Item = T>) -> crate::Result<()> {
    let iter = ii.into_iter();
    let _ = self.reserve_front(L::from_usize(iter.size_hint().0)?)?;
    for elem in iter {
      self.push_front(elem)?;
    }
    Ok(())
  }

  /// Provides a reference to the element at the given index.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// assert_eq!(queue.get(0), Some(&1));
  /// ```
  #[inline]
  pub fn get(&self, mut idx: L) -> Option<&T> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add_idx(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` points to valid memory
    let rslt = unsafe { self.data.as_ptr().add(idx.usize()) };
    // SAFETY: `idx` points to valid memory
    unsafe { Some(&*rslt) }
  }

  /// Mutable version of [`Self::get`].
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// assert_eq!(queue.get_mut(0), Some(&mut 1));
  /// ```
  #[inline]
  pub fn get_mut(&mut self, mut idx: L) -> Option<&mut T> {
    if idx >= self.data.len() {
      return None;
    }
    idx = wrap_add_idx(self.data.capacity(), self.head, idx);
    // SAFETY: `idx` points to valid memory
    let rslt = unsafe { self.data.as_ptr_mut().add(idx.usize()) };
    // SAFETY: `idx` points to valid memory
    unsafe { Some(&mut *rslt) }
  }

  /// Returns a front-to-back iterator.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
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
  /// let mut queue = wtx::collections::DequeU8::new();
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

  /// Indicates whether the internal data as organized as a contiguous slice of memory.
  #[inline]
  pub fn is_wrapping(&self) -> bool {
    is_wrapping(self.head, self.data.len(), self.tail)
  }

  /// Returns the last element.
  #[inline]
  pub fn last(&self) -> Option<&T> {
    self.get(self.len().checked_sub(L::ONE)?)
  }

  /// Returns the last mutable element.
  #[inline]
  pub fn last_mut(&mut self) -> Option<&mut T> {
    self.get_mut(self.len().checked_sub(L::ONE)?)
  }

  /// Returns the number of elements.
  #[inline]
  pub fn len(&self) -> L {
    self.data.len()
  }

  /// Removes the last element from the queue and returns it, or `None` if it is empty.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// queue.pop_back();
  /// assert_eq!(queue.as_slices(), (&[1][..], &[][..]));
  /// ```
  #[inline]
  pub fn pop_back(&mut self) -> Option<T> {
    let new_len = self.data.len().checked_sub(L::ONE)?;
    let curr_tail = wrap_sub_idx(self.data.capacity(), self.tail, L::ONE);
    self.tail = curr_tail;
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    // SAFETY: `idx` points to valid memory
    let src = unsafe { self.data.as_ptr_mut().add(curr_tail.usize()) };
    // SAFETY: `src` points to valid memory
    Some(unsafe { ptr::read(src) })
  }

  /// Removes the first element and returns it, or [`Option::None`] if the queue is empty.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// queue.pop_front();
  /// assert_eq!(queue.as_slices(), (&[3][..], &[][..]));
  /// ```
  #[inline]
  pub fn pop_front(&mut self) -> Option<T> {
    let new_len = self.data.len().checked_sub(L::ONE)?;
    let prev_head = self.head;
    self.head = wrap_add_idx(self.data.capacity(), prev_head, L::ONE);
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    // SAFETY: `prev_head` points to valid memory
    let src = unsafe { self.data.as_ptr_mut().add(prev_head.usize()) };
    // SAFETY: `src` points to valid memory
    Some(unsafe { ptr::read(src) })
  }

  /// Appends an element to the back of the queue.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_back(1);
  /// queue.push_back(3);
  /// assert_eq!(queue.as_slices(), (&[1, 3][..], &[][..]));
  /// ```
  #[inline]
  pub fn push_back(&mut self, value: T) -> crate::Result<()> {
    let _ = self.reserve_back(L::ONE).map_err(|_err| DequeueError::PushFrontOverflow)?;
    let len = self.data.len();
    let tail = wrap_idx(self.data.capacity(), self.tail);
    self.tail = wrap_add_idx(self.data.capacity(), self.tail, L::ONE);
    // SAFETY: `tail` is within bounds
    let dst = unsafe { self.data.as_ptr_mut().add(tail.usize()) };
    // SAFETY: `dst` points to valid memory
    unsafe {
      ptr::write(dst, value);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(len.wrapping_add(L::ONE));
    }
    Ok(())
  }

  /// Prepends an element to the queue.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_front(1);
  /// queue.push_front(3);
  /// assert_eq!(queue.as_slices(), (&[3, 1][..], &[][..]));
  /// ```
  #[inline]
  pub fn push_front(&mut self, value: T) -> crate::Result<()> {
    let _ = self.reserve_front(L::ONE).map_err(|_err| DequeueError::PushFrontOverflow)?;
    let len = self.data.len();
    self.head = wrap_sub_idx(self.data.capacity(), self.head, L::ONE);
    // SAFETY: `self.head` points to valid memory
    let dst = unsafe { self.data.as_ptr_mut().add(self.head.usize()) };
    // SAFETY: `dst` points to valid memory
    unsafe {
      ptr::write(dst, value);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(len.wrapping_add(L::ONE));
    }
    Ok(())
  }

  /// Reserves capacity for at least additional more elements to be inserted at the back of the
  /// queue.
  #[inline(always)]
  pub fn reserve_back(&mut self, additional: L) -> crate::Result<L> {
    let rr = self.prolong_back(additional)?;
    Ok(rr.head_shift)
  }

  /// Reserves capacity for at least additional more elements to be inserted at the front of the
  /// queue.
  #[inline(always)]
  pub fn reserve_front(&mut self, additional: L) -> crate::Result<L> {
    let rr = self.prolong_front(additional)?;
    Ok(rr.head_shift)
  }

  /// Shortens the queue, keeping the first `new_len` elements.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
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
  pub fn truncate_back(&mut self, new_len: L) {
    let _rslt = self.truncate_back_to_buffer(&mut (), new_len);
  }

  /// See [`Self::truncate_back`]. Transfers elements to `buffer` instead of dropping them.
  #[inline]
  pub fn truncate_back_to_buffer<B>(&mut self, buffer: &mut B, new_len: L) -> crate::Result<()>
  where
    B: TryExtend<[T; 1]>,
  {
    let len = self.data.len();
    let diff = if let Some(elem) = len.checked_sub(new_len)
      && elem > L::ZERO
    {
      elem
    } else {
      return Ok(());
    };
    if is_wrapping(self.head, len, self.tail) {
      if let Some(back_begin) = self.tail.checked_sub(diff) {
        if Self::NEEDS_DROP {
          // SAFETY: indices are within bounds
          unsafe {
            drop_elements(buffer, diff, back_begin, self.data.as_ptr_mut())?;
          }
        }
        self.tail = if self.tail == diff { L::ZERO } else { back_begin };
      } else {
        let front_len = diff.wrapping_sub(self.tail);
        let front_begin = self.data.capacity().wrapping_sub(front_len);
        if Self::NEEDS_DROP {
          // SAFETY: indices are within bounds
          unsafe {
            drop_elements(buffer, self.tail, L::ZERO, self.data.as_ptr_mut())?;
          }
          // SAFETY: indices are within bounds
          unsafe {
            drop_elements(buffer, front_len, front_begin, self.data.as_ptr_mut())?;
          }
        }
        self.tail = front_begin;
      }
    } else {
      let curr_tail = self.tail.wrapping_sub(diff);
      if Self::NEEDS_DROP {
        // SAFETY: indices are within bounds
        unsafe {
          drop_elements(buffer, diff, curr_tail, self.data.as_ptr_mut())?;
        }
      }
      self.tail = curr_tail;
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    Ok(())
  }

  /// Shortens the queue, keeping the last `new_len` elements.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
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
  pub fn truncate_front(&mut self, new_len: L) {
    let _rslt = self.truncate_front_to_buffer(&mut (), new_len);
  }

  /// See [`Self::truncate_front`]. Transfers elements to `buffer` instead of dropping them.
  #[inline]
  pub fn truncate_front_to_buffer<B>(&mut self, buffer: &mut B, new_len: L) -> crate::Result<()>
  where
    B: TryExtend<[T; 1]>,
  {
    let len = self.data.len();
    let diff = if let Some(elem) = len.checked_sub(new_len)
      && elem > L::ZERO
    {
      elem
    } else {
      return Ok(());
    };
    if is_wrapping(self.head, len, self.tail) {
      let front_slots = self.data.capacity().wrapping_sub(self.head);
      if front_slots >= diff {
        if Self::NEEDS_DROP {
          // SAFETY: indices are within bounds
          unsafe {
            drop_elements(buffer, diff, self.head, self.data.as_ptr_mut())?;
          }
        }
        self.head = if front_slots == diff { L::ZERO } else { self.head.wrapping_add(diff) }
      } else {
        let back_len = diff.wrapping_sub(front_slots);
        if Self::NEEDS_DROP {
          // SAFETY: indices are within bounds
          unsafe {
            drop_elements(buffer, front_slots, self.head, self.data.as_ptr_mut())?;
          }
          // SAFETY: indices are within bounds
          unsafe {
            drop_elements(buffer, back_len, L::ZERO, self.data.as_ptr_mut())?;
          }
        }
        self.head = back_len;
      }
    } else {
      let prev_head = self.head;
      if Self::NEEDS_DROP {
        // SAFETY: indices are within bounds
        unsafe {
          drop_elements(buffer, diff, prev_head, self.data.as_ptr_mut())?;
        }
      }
      self.head = prev_head.wrapping_add(diff);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(new_len);
    }
    Ok(())
  }

  pub(crate) const fn head(&self) -> L {
    self.head
  }

  unsafe fn expand(&mut self, additional: L, begin: L, new_len: L, value: T)
  where
    T: Clone,
  {
    // SAFETY: it is up to the caller to pass valid elements and enough allocated capacity
    let ptr = unsafe { self.data.as_ptr_mut().add(begin.usize()) };
    // SAFETY: it is up to the caller to pass valid elements and enough allocated capacity
    unsafe {
      slice::from_raw_parts_mut(ptr, additional.usize()).fill(value);
    }
    // SAFETY: it is up to the caller to pass valid elements and enough allocated capacity
    unsafe {
      self.data.set_len(new_len);
    }
  }

  fn prolong_back(&mut self, additional: L) -> crate::Result<ReserveRslt<L>> {
    reserve::<_, _, true>(additional, &mut self.data, &mut self.head, &mut self.tail)
  }

  fn prolong_front(&mut self, additional: L) -> crate::Result<ReserveRslt<L>> {
    reserve::<_, _, false>(additional, &mut self.data, &mut self.head, &mut self.tail)
  }

  fn slices_len<'iter>(iter: impl Iterator<Item = &'iter [T]>) -> crate::Result<L>
  where
    L: LinearStorageLen,
    T: 'iter,
  {
    let mut len: usize = 0;
    for other in iter {
      len = len.wrapping_add(other.len());
    }
    L::from_usize(len)
  }
}

impl<L, T> Deque<L, T>
where
  L: LinearStorageLen,
  T: Copy,
{
  /// Iterates over the `others` slices, copies each element, and then appends
  /// them to this instance. `others` are traversed in-order.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_back(4);
  /// queue.extend_back_from_copyable_slices([&[2, 3][..]]);
  /// queue.extend_back_from_copyable_slices([&[0, 1][..], &[1][..]]);
  /// assert_eq!(queue.as_slices(), (&[4, 2, 3, 0, 1, 1][..], &[][..]));
  /// ```
  #[inline]
  pub fn extend_back_from_copyable_slices<'iter, I>(&mut self, others: I) -> crate::Result<L>
  where
    I: IntoIterator<Item = &'iter [T]>,
    I::IntoIter: Clone,
    T: 'iter,
  {
    let iter = others.into_iter();
    let others_len = Self::slices_len(iter.clone())?;
    let rr = self.prolong_back(others_len)?;
    let mut shift = rr.begin;
    for other in iter {
      // SAFETY: `self.head` points to valid memory
      let dst = unsafe { self.data.as_ptr_mut().add(shift.usize()) };
      // SAFETY: `dst` points to valid memory
      unsafe {
        ptr::copy_nonoverlapping(other.as_ptr(), dst, other.len());
      }
      shift = shift.wrapping_add(L::from_usize(other.len())?);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(self.data.len().wrapping_add(others_len));
    }
    self.tail = rr.begin.wrapping_add(others_len);
    Ok(others_len)
  }

  /// Iterates over the `others` slices, copies each element, and then prepends
  /// them to this instance. `others` are traversed in-order.
  ///
  /// ```rust
  /// let mut queue = wtx::collections::DequeU8::new();
  /// queue.push_front(4);
  /// queue.extend_front_from_copyable_slices([&[2, 3][..]]);
  /// queue.extend_front_from_copyable_slices([&[0, 1][..], &[1][..]]);
  /// assert_eq!(queue.as_slices(), (&[0, 1, 1, 2, 3, 4][..], &[][..]));
  /// ```
  #[inline]
  pub fn extend_front_from_copyable_slices<'iter, I>(&mut self, others: I) -> crate::Result<L>
  where
    I: IntoIterator<Item = &'iter [T]>,
    I::IntoIter: Clone,
    T: 'iter,
  {
    let iter = others.into_iter();
    let others_len = Self::slices_len(iter.clone())?;
    let rr = self.prolong_front(others_len)?;
    let mut shift = rr.begin;
    for other in iter {
      // SAFETY: `shift` points to newly allocated memory
      let dst = unsafe { self.data.as_ptr_mut().add(shift.usize()) };
      // SAFETY: `dst` points to valid memory
      unsafe {
        ptr::copy_nonoverlapping(other.as_ptr(), dst, other.len());
      }
      shift = shift.wrapping_add(L::from_usize(other.len())?);
    }
    // SAFETY: is within bounds
    unsafe {
      self.data.set_len(self.data.len().wrapping_add(others_len));
    }
    self.head = rr.begin;
    Ok(others_len)
  }
}

impl<L, T> Clone for Deque<L, T>
where
  L: LinearStorageLen,
  T: Clone,
{
  #[inline]
  fn clone(&self) -> Self {
    let mut instance = Deque::new();
    let _rslt = instance.extend_back_from_iter(self.iter().cloned());
    instance
  }
}

impl<L, T> Debug for Deque<L, T>
where
  L: LinearStorageLen,
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    let (front, back) = self.as_slices();
    f.debug_struct("Deque").field("front", &front).field("back", &back).finish()
  }
}

impl<L, T> Default for Deque<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<L, T> Drop for Deque<L, T>
where
  L: LinearStorageLen,
{
  #[inline]
  fn drop(&mut self) {
    struct Guard<'any, T>(&'any mut [T]);
    impl<T> Drop for Guard<'_, T> {
      fn drop(&mut self) {
        // SAFETY: `as_slices_mut` returns initialized elements
        unsafe {
          ptr::drop_in_place(self.0);
        }
      }
    }

    {
      let (front, back) = self.as_slices_mut();
      let _back_dropper = Guard(back);
      let _front_dropper = Guard(front);
    }

    // SAFETY: We have just manually dropped all valid elements.
    unsafe {
      self.data.set_len(L::ZERO);
    }
  }
}

struct ReserveRslt<L> {
  /// Starting index where the `additional` number of elements can be inserted.
  begin: L,
  /// The number os places the head must be shift.
  head_shift: L,
}

impl<L> ReserveRslt<L>
where
  L: LinearStorageLen,
{
  fn new(begin: L, head_shift: L) -> Self {
    Self { begin, head_shift }
  }
}

/// All wrapping structures must have at least 1 element.
///
/// ```txt
/// H = Head (inclusive)
/// T = Tail (exclusive)
///
/// ***** T(8) DOES NOT EXIST *****
///
/// # No wrapping
///
/// H(0) = T(0): . . . . . . . . (no wrapping)
/// H(1) = T(1): . . . . . . . . (no wrapping)
///
/// H(0) < T(1): H . . . . . . . (no wrapping)
/// H(0) < T(2): H T . . . . . . (no wrapping)
/// H(3) < T(6): . . . H * T . . (no wrapping)
///
/// # Wrapping
///
/// H(0) = T(0): H * * * * * * T (wrapping)
/// H(2) = T(2): * T H * * * * * (wrapping)
/// H(7) = T(7): * * * * * * T H (wrapping)
///
/// H(6) > T(0): . . . . . . H * (wrapping)
/// H(6) > T(4): * * * T . . H * (wrapping)
/// H(7) > T(0): . . . . . . . H (wrapping)
/// H(7) > T(1): T . . . . . . H (wrapping)
/// ```
fn is_wrapping<L>(head: L, len: L, tail: L) -> bool
where
  L: LinearStorageLen,
{
  if tail > head { false } else { len > L::ZERO }
}

/// Allocates `additional` capacity for the contiguous insertion of back or front elements. This
/// also means that the free capacity of intersections is not considered.
#[inline(always)]
fn reserve<L, T, const IS_BACK: bool>(
  additional: L,
  data: &mut Vector<L, T>,
  head: &mut L,
  tail: &mut L,
) -> crate::Result<ReserveRslt<L>>
where
  L: LinearStorageLen,
{
  let len = data.len();
  let prev_cap = data.capacity();
  data.reserve(additional).map_err(|_err| DequeueError::ReserveOverflow)?;
  let curr_cap = data.capacity();
  let prev_head = prev_cap.min(*head);
  let prev_tail = prev_cap.min(*tail);
  if len == L::ZERO {
    *head = L::ZERO;
    *tail = L::ZERO;
    return Ok(if IS_BACK {
      ReserveRslt::new(L::ZERO, L::ZERO)
    } else {
      ReserveRslt::new(curr_cap.wrapping_sub(additional), L::ZERO)
    });
  }
  if is_wrapping(prev_head, len, prev_tail) {
    let free_slots = prev_head.wrapping_sub(prev_tail);
    if free_slots >= additional {
      return Ok(if IS_BACK {
        ReserveRslt::new(prev_tail, L::ZERO)
      } else {
        ReserveRslt::new(prev_head.wrapping_sub(additional), L::ZERO)
      });
    }
    let front_len = prev_cap.wrapping_sub(prev_head);
    let curr_head = curr_cap.wrapping_sub(front_len);
    // SAFETY: `prev_head` is equal or less than the current capacity
    let src = unsafe { data.as_ptr_mut().add(prev_head.usize()) };
    // SAFETY: `curr_head` is equal or less than the current capacity
    let dst = unsafe { data.as_ptr_mut().add(curr_head.usize()) };
    // SAFETY: memory has been allocated
    unsafe {
      ptr::copy(src, dst, front_len.usize());
    }
    *head = curr_head;
    Ok(if IS_BACK {
      ReserveRslt::new(prev_tail, L::ZERO)
    } else {
      ReserveRslt::new(curr_head.saturating_sub(additional), curr_cap.wrapping_sub(prev_cap))
    })
  } else {
    let left_free = prev_head;
    let right_free = curr_cap.wrapping_sub(prev_tail);
    if IS_BACK {
      if right_free >= additional {
        return Ok(ReserveRslt::new(prev_tail, L::ZERO));
      }
      if right_free == L::ZERO && left_free >= additional {
        return Ok(ReserveRslt::new(L::ZERO, L::ZERO));
      }
      // SAFETY: `prev_head` is equal or less than the current capacity
      let src = unsafe { data.as_ptr_mut().add(prev_head.usize()) };
      // SAFETY: memory has been allocated
      unsafe {
        ptr::copy(src, data.as_ptr_mut(), len.usize());
      }
      let curr_tail = len;
      *head = L::ZERO;
      *tail = curr_tail;
      Ok(ReserveRslt::new(curr_tail, L::ZERO))
    } else {
      if left_free >= additional {
        return Ok(ReserveRslt::new(prev_head.wrapping_sub(additional), L::ZERO));
      }
      if left_free == L::ZERO && right_free >= additional {
        return Ok(ReserveRslt::new(curr_cap.wrapping_sub(additional), L::ZERO));
      }
      let curr_head = curr_cap.wrapping_sub(len);
      // SAFETY: `prev_head` is equal or less than the current capacity
      let src = unsafe { data.as_ptr_mut().add(prev_head.usize()) };
      // SAFETY: `curr_head` is equal or less than the current capacity
      let dst = unsafe { data.as_ptr_mut().add(curr_head.usize()) };
      // SAFETY: memory has been allocated
      unsafe {
        ptr::copy(src, dst, len.usize());
      }
      *head = curr_head;
      *tail = curr_cap;
      Ok(ReserveRslt::new(curr_head.wrapping_sub(additional), right_free))
    }
  }
}

fn wrap_add_idx<L>(bound: L, idx: L, offset: L) -> L
where
  L: LinearStorageLen,
{
  wrap_idx(bound, idx.wrapping_add(offset))
}

fn wrap_idx<L>(bound: L, idx: L) -> L
where
  L: LinearStorageLen,
{
  idx.checked_sub(bound).unwrap_or(idx)
}

fn wrap_sub_idx<L>(bound: L, idx: L, offset: L) -> L
where
  L: LinearStorageLen,
{
  wrap_idx(bound, idx.wrapping_sub(offset).wrapping_add(bound))
}
