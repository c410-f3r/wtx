#![allow(clippy::as_conversions, reason = "address is only used as an heuristic to retrieve locks")]

use crate::misc::{CachePadded, sync::seq_lock::SeqLock};
use core::{
  cell::UnsafeCell,
  fmt::{Debug, Formatter},
  mem,
  panic::{RefUnwindSafe, UnwindSafe},
  ptr,
};

const LEN: usize = 67;

static LOCKS: [CachePadded<SeqLock>; LEN] = [const { CachePadded(SeqLock::new()) }; LEN];

/// A type that allows copyable elements to be safely shared between threads.
pub struct AtomicCell<T> {
  value: UnsafeCell<T>,
}

impl<T> AtomicCell<T> {
  /// Creates a new instance.
  #[inline]
  pub const fn new(value: T) -> AtomicCell<T> {
    AtomicCell { value: UnsafeCell::new(value) }
  }

  /// Returns inner data.
  ///
  /// ```rust
  /// let ac = wtx::misc::AtomicCell::new(7);
  /// assert_eq!(ac.into_inner(), 7);
  /// ```
  #[inline]
  pub fn into_inner(self) -> T {
    self.value.into_inner()
  }

  /// Loads a value from the atomic cell.
  ///
  /// ```rust
  /// let ac = wtx::misc::AtomicCell::new(7);
  /// assert_eq!(ac.load(), 7);
  /// ```
  #[inline]
  pub fn load(&self) -> T
  where
    T: Copy,
  {
    let src = self.as_ptr();
    // FIXME(STABLE): strict_provenance
    let lock = lock(src as usize);

    if let Some(stamp) = lock.optimistic_read() {
      // SAFETY: pointer doesn't have offsets
      let value = unsafe { ptr::read_volatile(src) };
      if lock.validate_read(stamp) {
        return value;
      }
    }

    let guard = lock.write();
    // SAFETY: pointer doesn't have offsets
    let value = unsafe { ptr::read(src) };
    guard.abort();
    value
  }

  /// Stores `value` into the atomic cell.
  ///
  /// ```
  /// let ac = wtx::misc::AtomicCell::new(7);
  /// assert_eq!(ac.load(), 7);
  /// ac.store(8);
  /// assert_eq!(ac.load(), 8);
  /// ```
  #[inline]
  pub fn store(&self, value: T) {
    if const { mem::needs_drop::<T>() } {
      drop(self.swap(value));
    } else {
      let dst = self.as_ptr();
      // FIXME(STABLE): strict_provenance
      let _guard = lock(dst as usize).write();
      // SAFETY: pointer doesn't have offsets and `value` has the same size and alignment of `self`
      unsafe {
        ptr::write(dst, value);
      }
    }
  }

  /// Stores `value` into the atomic cell and returns the previous value.
  ///
  /// ```
  /// let ac = wtx::misc::AtomicCell::new(7);
  /// assert_eq!(ac.load(), 7);
  /// assert_eq!(ac.swap(8), 7);
  /// assert_eq!(ac.load(), 8);
  /// ```
  #[inline]
  pub fn swap(&self, value: T) -> T {
    let dst = self.value.get();
    // FIXME(STABLE): strict_provenance
    let _guard = lock(dst as usize).write();
    // SAFETY: pointer doesn't have offsets and `value` has the same size and alignment of `self`
    unsafe { ptr::replace(dst, value) }
  }

  #[inline]
  fn as_ptr(&self) -> *mut T {
    self.value.get()
  }
}

impl<T> Debug for AtomicCell<T>
where
  T: Copy + Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("AtomicCell").field("value", &self.load()).finish()
  }
}

impl<T> Default for AtomicCell<T>
where
  T: Default,
{
  #[inline]
  fn default() -> AtomicCell<T> {
    AtomicCell::new(T::default())
  }
}

impl<T> From<T> for AtomicCell<T> {
  #[inline]
  fn from(value: T) -> AtomicCell<T> {
    AtomicCell::new(value)
  }
}

impl<T> RefUnwindSafe for AtomicCell<T> {}

// SAFETY: Concurrent access is manually managed
unsafe impl<T> Send for AtomicCell<T> where T: Send {}

// SAFETY: Concurrent access is manually managed
unsafe impl<T: Send> Sync for AtomicCell<T> {}

impl<T> UnwindSafe for AtomicCell<T> where T: Send {}

#[inline]
fn lock(addr: usize) -> &'static SeqLock {
  #[allow(clippy::indexing_slicing, reason = "modulo result will always be in-bounds")]
  &LOCKS[addr % LEN].0
}

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::AtomicCell;
  use serde::{Serialize, Serializer};

  impl<T> Serialize for AtomicCell<T>
  where
    T: Copy + Serialize,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      T::serialize(&self.load(), serializer)
    }
  }
}
