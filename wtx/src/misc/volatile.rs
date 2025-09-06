use core::ptr;

/// Sets the first `len` elements pointed by `dst` according to the specified `src`.
///
/// Volatile operations are intended to act on I/O memory, and are guaranteed to not be elided or
/// reordered by the compiler across other volatile operations.
///
/// # Safety
///
/// `dst` must be a valid continuous block of allocated memory with at least `len` elements.
#[inline(never)]
pub unsafe fn memset_volatile<T>(dst: *mut T, src: T, len: usize)
where
  T: Copy + Sized,
{
  for idx in 0..len {
    // Safety: up to the caller
    let ptr = unsafe { dst.add(idx) };
    // Safety: up to the caller
    unsafe {
      ptr::write_volatile(ptr, src);
    }
  }
}

/// Fills `slice` with elements through the copying of `src`.
///
/// Volatile operations are intended to act on I/O memory, and are guaranteed to not be elided or
/// reordered by the compiler across other volatile operations.
#[inline]
pub fn memset_slice_volatile<T>(slice: &mut [T], src: T)
where
  T: Copy + Sized,
{
  let len = slice.len();
  let dst = slice.as_mut_ptr().cast();
  // SAFETY: parameters are originated from an existing slice
  unsafe {
    memset_volatile(dst, src, len);
  }
}
