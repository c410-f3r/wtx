use crate::{misc::SensitiveBytes, secret::SecretData};
use alloc::string::String;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
  ptr::{self, NonNull},
  slice, str,
};
use libc::O_CLOEXEC;

/// `memfd_secret`
pub struct MemFdSecret {
  len: usize,
  ptr: NonNull<u8>,
}

impl MemFdSecret {
  /// New instance
  #[inline]
  pub fn new(bytes: &mut [u8], cloexec: bool) -> Option<Self> {
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

/// A [`MemFdSecret`] that represents a fixed-size array of secret bytes.
#[derive(Debug)]
#[repr(transparent)]
pub struct MemFdSecretArray<const N: usize>(MemFdSecret);
impl<const N: usize> SecretData for MemFdSecretArray<N> {
  type ReprSrc<'this> = MemFdSecretArrayReprSrc<'this, N>;

  #[inline]
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    Ok(Self(
      MemFdSecret::new(data.as_mut_slice(), true).ok_or(crate::Error::UnsupportedLinuxKernel)?,
    ))
  }

  #[inline]
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>> {
    Ok(MemFdSecretArrayReprSrc(&self.0))
  }
}
/// Temporary plaintext guard for [`MemFdSecretArray`].
#[derive(Debug)]
pub struct MemFdSecretArrayReprSrc<'sd, const N: usize>(&'sd MemFdSecret);
impl<const N: usize> Deref for MemFdSecretArrayReprSrc<'_, N> {
  type Target = [u8; N];

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: The bytes in this instance have the same layout of the bytes used when constructing
    unsafe { self.0.as_array().unwrap_unchecked() }
  }
}

/// A [`MemFdSecret`] that represents a dynamically-sized slice of secret bytes.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct MemFdSecretSlice(MemFdSecret);
impl SecretData for MemFdSecretSlice {
  type ReprSrc<'this> = MemFdSecretSliceReprSrc<'this>;

  #[inline]
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    Ok(Self(MemFdSecret::new(data, true).ok_or(crate::Error::UnsupportedLinuxKernel)?))
  }

  #[inline]
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>> {
    Ok(MemFdSecretSliceReprSrc(&self.0))
  }
}
/// Temporary plaintext guard for [`MemFdSecretSlice`].
#[derive(Debug)]
pub struct MemFdSecretSliceReprSrc<'sd>(&'sd MemFdSecret);
impl Deref for MemFdSecretSliceReprSrc<'_> {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0
  }
}

/// A [`MemFdSecret`] that represents a valid UTF-8 secret string.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct MemFdSecretStr(MemFdSecret);
impl SecretData for MemFdSecretStr {
  type ReprSrc<'this> = MemFdSecretStrReprSrc<'this>;

  #[inline]
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    // SAFETY: `data` is zeroed by `MemFdSecret` and since are zeros are ASCII, the conversion
    //         is safe.
    let bytes = unsafe { data.as_bytes_mut() };
    Ok(Self(MemFdSecret::new(bytes, true).ok_or(crate::Error::UnsupportedLinuxKernel)?))
  }

  #[inline]
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>> {
    Ok(MemFdSecretStrReprSrc(&self.0))
  }
}
/// Temporary plaintext guard for [`MemFdSecretStr`].
#[derive(Debug)]
pub struct MemFdSecretStrReprSrc<'sd>(&'sd MemFdSecret);
impl Deref for MemFdSecretStrReprSrc<'_> {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: The bytes of this instance have the same layout of the bytes used when constructing
    unsafe { str::from_utf8_unchecked(self.0) }
  }
}
impl TryFrom<String> for MemFdSecretStr {
  type Error = crate::Error;

  #[inline]
  fn try_from(mut value: String) -> crate::Result<Self> {
    Self::new(value.as_mut_str())
  }
}
