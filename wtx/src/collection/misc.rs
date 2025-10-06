use crate::collection::LinearStorageLen;
use core::{ptr, slice};

/// Calculates the current index of a previous `push_front` element.
///
/// `elem_idx` and `elem_idx_last` should be orchestrated by an index that is always increasing
/// at each `push_front` invocation.
///
/// ```rust
/// use wtx::collection::{Deque, backward_deque_idx};
///
/// let mut deque = Deque::new();
///
/// let idx_of_first_elem = 0;
/// deque.push_front(1).unwrap();
/// assert_eq!(deque.get(backward_deque_idx(idx_of_first_elem, idx_of_first_elem)), Some(&1));
///
/// let idx_of_second_elem = 1;
/// deque.push_front(2).unwrap();
/// assert_eq!(deque.get(backward_deque_idx(idx_of_first_elem, idx_of_second_elem)), Some(&1));
/// assert_eq!(deque.get(backward_deque_idx(idx_of_second_elem, idx_of_second_elem)), Some(&2));
///
/// let idx_of_third_elem = 2;
/// deque.push_front(3).unwrap();
/// assert_eq!(deque.get(backward_deque_idx(idx_of_first_elem, idx_of_third_elem)), Some(&1));
/// assert_eq!(deque.get(backward_deque_idx(idx_of_second_elem, idx_of_third_elem)), Some(&2));
/// assert_eq!(deque.get(backward_deque_idx(idx_of_third_elem, idx_of_third_elem)), Some(&3));
/// ```
#[inline]
pub const fn backward_deque_idx(elem_idx: usize, elem_idx_last: usize) -> usize {
  elem_idx_last.wrapping_sub(elem_idx) % 9_223_372_036_854_775_807
}

pub(crate) unsafe fn drop_elements<L, T>(len: L, offset: L, ptr: *mut T)
where
  L: LinearStorageLen,
{
  // SAFETY: it is up to the caller to provide a valid pointer with a valid index
  let data = unsafe { ptr.add(offset.usize()) };
  // SAFETY: it is up to the caller to provide a valid length
  let elements = unsafe { slice::from_raw_parts_mut(data, len.usize()) };
  // SAFETY: it is up to the caller to provide parameters that can lead to droppable elements
  unsafe {
    ptr::drop_in_place(elements);
  }
}

pub(crate) fn is_char_boundary(idx: usize, slice: &[u8]) -> bool {
  if idx == 0 {
    return true;
  }
  if idx >= slice.len() {
    idx == slice.len()
  } else {
    let byte = slice.get(idx).copied().unwrap_or_default();
    !(128..192).contains(&byte)
  }
}

//A deque can only store up to 4 elements represented by an integer of 2 bits. The head index is also represented by an integer of 2 bits.
//
//Everytime the `push_front` method is called, an auxiliar `added_index` integer variable of 2 bits is used to keep track of mutable indices of
//previous elements.
//
//In a scenario of 3 `push_front`. How to calculate the `mutable_index` of all existing elements?
//added_index    = x 2 1 0
//mutable_index  = x 0 1 2
//
//In a scenario of 4 `push_front` and 1 `pop_back`. How to calculate the `mutable_index` of all existing elements?
//added_index    = 3 2 1 x
//mutable_index  = 0 1 2 x
//
//In a scenario of 5 `push_front` and 3 `pop_back`. How to calculate the `mutable_index` of all existing elements?
//added_index    = 3 - - 0
//mutable_index  = 1 - - 0
//
//How to calculate the `mutable_index` of all existing elements regardless of the scenario?
