use crate::misc::{Vector, _shift_bytes};
use core::{iter, ptr};

#[inline(always)]
pub(crate) fn reserve<D>(additional: usize, data: &mut Vector<D>, head: &mut usize) -> Option<usize>
where
  D: Copy,
{
  let prev_cap = data.capacity();
  let prev_head = *head;
  let rhs_len = prev_cap.wrapping_sub(prev_head);
  data.reserve(additional);
  let curr_cap = data.capacity();
  let cap_diff = curr_cap.wrapping_sub(prev_cap);
  if prev_cap == 0 || cap_diff == 0 {
    return None;
  }
  let curr_head = curr_cap.wrapping_sub(rhs_len);
  // SAFETY: Slice is allocated but not initialized
  let allocated = unsafe {
    let rslt = &mut *ptr::slice_from_raw_parts_mut(data.as_mut_ptr(), curr_cap);
    #[cfg(feature = "nightly")]
    {
      core::hint::assert_unchecked(curr_head <= rslt.len());
      core::hint::assert_unchecked(prev_head <= prev_cap);
    }
    rslt
  };
  let _ = _shift_bytes(curr_head, allocated, iter::once(prev_head..prev_cap));
  *head = curr_head;
  Some(cap_diff)
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