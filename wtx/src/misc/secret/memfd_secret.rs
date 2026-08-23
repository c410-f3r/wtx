use crate::misc::SensitiveBytes;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
  ptr::{self, NonNull},
  slice,
};
use libc::O_CLOEXEC;

pub(crate) struct MemFdSecret {
  len: usize,
  ptr: NonNull<u8>,
}

impl MemFdSecret {
  #[inline]
  pub(crate) fn new(bytes: &mut [u8], cloexec: bool) -> Option<Self> {
    struct FdGuard(libc::c_int);

    impl Drop for FdGuard {
      fn drop(&mut self) {
        // SAFETY: Only called after a syscall was performed
        unsafe {
          let _ = libc::close(self.0);
        }
      }
    }

    let bytes_sb = SensitiveBytes::new(bytes);
    let len = bytes_sb.len();
    if len == 0 {
      return Some(Self::default());
    }
    // SAFETY: This module is only accessible in linux hosts
    let fd = unsafe {
      let arg = if cloexec { O_CLOEXEC } else { 0 };
      libc::syscall(libc::SYS_memfd_secret, arg)
    };
    if fd == -1 {
      return None;
    }
    let fd_int: libc::c_int = fd.try_into().ok()?;
    let _guard = FdGuard(fd_int);
    // SAFETY: `fd_int` originates from the newly allocated `fd`
    if unsafe { libc::ftruncate(fd_int, len.try_into().ok()?) } != 0 {
      return None;
    }
    // SAFETY: `fd_int` originates from the newly allocated `fd`
    let dst = unsafe {
      libc::mmap(
        ptr::null_mut(),
        len,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED,
        fd_int,
        0,
      )
    };
    if dst == libc::MAP_FAILED {
      return None;
    }
    let dst_ptr = dst.cast::<u8>();
    // SAFETY: `dst_ptr` is a fresh valid mapping and both pointers have `len` bytes
    unsafe {
      ptr::copy_nonoverlapping(bytes_sb.as_ptr(), dst_ptr, len);
    }
    Some(Self {
      // Safety: `dst_ptr` is non-null because `mmap` succeeded
      ptr: unsafe { NonNull::new_unchecked(dst_ptr) },
      len,
    })
  }
}

impl Debug for MemFdSecret {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("MemFdSecret").finish()
  }
}

impl Default for MemFdSecret {
  #[inline]
  fn default() -> Self {
    Self { len: 0, ptr: NonNull::dangling() }
  }
}

impl Deref for MemFdSecret {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: length is exactly what was successfully mapped
    unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
  }
}

impl DerefMut for MemFdSecret {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    // SAFETY: length is exactly what was successfully mapped
    unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
  }
}

impl Drop for MemFdSecret {
  #[inline]
  fn drop(&mut self) {
    drop(SensitiveBytes::new(&mut **self));
    if self.len > 0 {
      // SAFETY: Instance was created with `mmap` using the same pointer and length
      unsafe {
        let _ = libc::munmap(self.ptr.as_ptr().cast(), self.len);
      }
    }
  }
}

// SAFETY: There is no method that gives mutable access in immutable contexts
unsafe impl Send for MemFdSecret {}

// SAFETY: There is no method that gives mutable access in immutable contexts
unsafe impl Sync for MemFdSecret {}
