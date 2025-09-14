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

/// Locks a chunk of data into RAM, preventing it from being paged to the swap area.
///
/// # Safety
///
/// - `addr` must point to `len` bytes of valid memory
#[inline]
pub unsafe fn mlock(_addr: *mut u8, _len: usize) -> crate::Result<()> {
  #[cfg(feature = "libc")]
  {
    // SAFETY: up to the caller
    let mlock = unsafe { libc::mlock(_addr.cast(), _len) };
    if mlock != 0 {
      return Err(crate::Error::MlockError);
    }
    Ok(())
  }
  #[cfg(not(feature = "libc"))]
  return Err(crate::Error::UnsupportedMlockPlatform);
}

/// Locks a chunk of bytes into RAM, preventing it from being paged to the swap area.
#[inline]
pub fn mlock_slice(_bytes: &mut [u8]) -> crate::Result<()> {
  // SAFETY: pointer comes from allocated memory
  #[cfg(not(miri))]
  unsafe {
    let len = _bytes.len();
    mlock(_bytes.as_mut_ptr(), len)?;
  }
  Ok(())
}

/// Unlocks memory flagged with [`mlock`].
///
/// # Safety
///
/// - `addr` must point to `len` bytes of valid memory
#[inline]
pub unsafe fn munlock(_addr: *mut u8, _len: usize) -> crate::Result<()> {
  #[cfg(feature = "libc")]
  {
    // SAFETY: up to the caller
    let munlock = unsafe { libc::munlock(_addr.cast(), _len) };
    if munlock != 0 {
      return Err(crate::Error::MlockError);
    }
    Ok(())
  }
  #[cfg(not(feature = "libc"))]
  return Err(crate::Error::UnsupportedMlockPlatform);
}

#[cfg(all(feature = "libc", test))]
mod tests {
  use crate::{
    collection::Vector,
    misc::{mlock, munlock},
  };

  #[test]
  fn mlock_and_munlock() {
    let mut data = Vector::with_capacity(1024).unwrap();
    unsafe {
      mlock(data.as_mut_ptr(), data.capacity()).unwrap();
    }
    unsafe {
      munlock(data.as_mut_ptr(), data.capacity()).unwrap();
    }
  }
}
