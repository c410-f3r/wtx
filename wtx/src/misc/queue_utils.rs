use crate::misc::{Vector, _shift_bytes};
use core::{hint::assert_unchecked, iter, ptr};

/// If memory is allocated, tail elements are shifted to the right.
///
/// ```ignore
/// |4|3|2|6|5|
///       |
///        Head
///
/// |4|3|2|6|5| | | | | | | | | |
///       |                 |
///        Previous head     Current head
///
/// |4|3|2| | | | | | | | | |6|5|
///                         |
///                          Head
/// ```
#[inline(always)]
pub(crate) fn reserve<D>(
  additional: usize,
  data: &mut Vector<D>,
  head: &mut usize,
) -> Result<Option<usize>, ()>
where
  D: Copy,
{
  let prev_cap = data.capacity();
  data.reserve(additional).map_err(|_err| ())?;
  let prev_head = *head;
  let rhs_len = prev_cap.wrapping_sub(prev_head);
  let curr_cap = data.capacity();
  let cap_diff = curr_cap.wrapping_sub(prev_cap);
  if data.len() == 0 {
    return Ok(None);
  }
  let curr_head = curr_cap.wrapping_sub(rhs_len);
  // SAFETY: memory has been allocated
  let allocated = unsafe { &mut *ptr::slice_from_raw_parts_mut(data.as_mut_ptr(), curr_cap) };
  // SAFETY: head will never be greater than the number of allocated elements
  unsafe {
    assert_unchecked(allocated.len() >= curr_head || prev_cap >= prev_head);
  }
  let _ = _shift_bytes(curr_head, allocated, iter::once(prev_head..prev_cap));
  *head = curr_head;
  Ok(Some(cap_diff))
}

#[inline]
pub(crate) fn wrap_add(cap: usize, idx: usize, value: usize) -> usize {
  wrap_idx(idx.wrapping_add(value), cap)
}

#[inline]
pub(crate) fn wrap_idx(idx: usize, cap: usize) -> usize {
  idx.checked_sub(cap).unwrap_or(idx)
}

#[inline]
pub(crate) fn wrap_sub(cap: usize, idx: usize, value: usize) -> usize {
  wrap_idx(idx.wrapping_sub(value).wrapping_add(cap), cap)
}
