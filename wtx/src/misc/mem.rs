/// Locks of chunk of data into RAM, preventing it from being paged to the swap area.
///
/// # Safety
///
/// - `addr` must point to `len` bytes of valid memory
#[inline]
pub unsafe fn mlock(_addr: *mut u8, _len: usize) -> crate::Result<()> {
  #[cfg(all(feature = "libc", target_os = "linux"))]
  {
    // SAFETY: up to the caller
    let mlock = unsafe { libc::mlock(_addr.cast(), _len) };
    if mlock != 0 {
      return Err(crate::Error::MlockError);
    }
    Ok(())
  }
  #[cfg(not(all(feature = "libc", target_os = "linux")))]
  return Err(crate::Error::UnsupportedMlockPlatform);
}

/// Unlocks memory flagged with [`mlock`].
///
/// # Safety
///
/// - `addr` must point to `len` bytes of valid memory
#[inline]
pub unsafe fn munlock(_addr: *mut u8, _len: usize) -> crate::Result<()> {
  #[cfg(all(feature = "libc", target_os = "linux"))]
  {
    // SAFETY: up to the caller
    let munlock = unsafe { libc::munlock(_addr.cast(), _len) };
    if munlock != 0 {
      return Err(crate::Error::MlockError);
    }
    Ok(())
  }
  #[cfg(not(all(feature = "libc", target_os = "linux")))]
  return Err(crate::Error::UnsupportedMlockPlatform);
}

#[cfg(all(feature = "libc", target_os = "linux", test))]
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
