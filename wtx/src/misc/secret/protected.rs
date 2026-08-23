use crate::misc::memset_slice_volatile;
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
};

// A chunk of heap-allocated memory that is zeroed when dropped. The use of a pointer
// prevents compiler optimizations
pub(crate) struct Protected(*mut [u8]);

impl Protected {
  #[inline]
  pub(crate) fn zeroed(size: usize) -> Protected {
    alloc::vec![0; size].into_boxed_slice().into()
  }
}

impl Debug for Protected {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Protected").finish()
  }
}

impl Deref for Protected {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    // SAFETY: Pointer comes from a valid owned chunk of memory according to all related
    //         constructors
    unsafe { &*self.0 }
  }
}

impl DerefMut for Protected {
  #[inline]
  fn deref_mut(&mut self) -> &mut [u8] {
    // SAFETY: Pointer comes from a valid owned chunk of memory according to all related
    //         constructors
    unsafe { &mut *self.0 }
  }
}

impl Drop for Protected {
  #[inline]
  fn drop(&mut self) {
    memset_slice_volatile(self, 0);
    #[cfg(feature = "libc")]
    let _rslt = crate::misc::munlock_slice(self);
    // SAFETY: Instance has a valid allocated chunk of memory
    unsafe {
      drop(Box::from_raw(self.0));
    }
  }
}

impl From<&[u8]> for Protected {
  #[inline]
  fn from(from: &[u8]) -> Self {
    let mut protected = Protected::zeroed(from.len());
    protected.copy_from_slice(from);
    protected
  }
}

impl From<Box<[u8]>> for Protected {
  #[inline]
  fn from(from: Box<[u8]>) -> Self {
    Protected(Box::into_raw(from))
  }
}

// SAFETY: Inner pointer is unique
unsafe impl Send for Protected {}
// SAFETY: Inner pointer is unique
unsafe impl Sync for Protected {}
