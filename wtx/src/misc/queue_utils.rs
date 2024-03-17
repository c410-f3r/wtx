use crate::misc::{Vector, _shift_bytes};
use core::ptr;

#[inline(always)]
pub(crate) fn reserve<D>(additional: usize, data: &mut Vector<D>, head: &mut usize)
where
  D: Copy,
{
  let prev_cap = data.capacity();
  let prev_head = *head;
  let rhs_len = prev_cap.wrapping_sub(prev_head);
  data.reserve(additional);
  if data.capacity() <= prev_cap {
    return;
  }
  let curr_head = data.capacity().wrapping_sub(rhs_len);
  let allocated = unsafe {
    let ptr = data.as_mut_ptr();
    &mut *ptr::slice_from_raw_parts_mut(ptr, data.capacity())
  };
  #[cfg(feature = "nightly")]
  unsafe {
    core::hint::assert_unchecked(curr_head <= allocated.len());
    core::hint::assert_unchecked(prev_head <= prev_cap);
  }
  _shift_bytes(curr_head, allocated, [prev_head..prev_cap]);
  *head = curr_head;
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
