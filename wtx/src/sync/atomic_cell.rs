use crate::sync::{CachePadded, SeqLock};
use core::{
  cell::UnsafeCell,
  fmt::{Debug, Formatter},
  mem::{self, ManuallyDrop},
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
  /// let ac = wtx::sync::AtomicCell::new(7);
  /// assert_eq!(ac.into_inner(), 7);
  /// ```
  #[inline]
  pub fn into_inner(self) -> T {
    let this = ManuallyDrop::new(self);
    // SAFETY:
    // - ownership prevents concurrent access and ensures a valid pointer
    // - `ManuallyDrop` prevents double free
    unsafe { this.as_ptr().read() }
  }

  /// Stores `value` into the atomic cell.
  ///
  /// ```
  /// let ac = wtx::sync::AtomicCell::new(7);
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
      let _guard = lock(dst.addr()).write();
      // SAFETY: pointer doesn't have offsets and `value` has the same size and alignment of `self`
      unsafe {
        ptr::write(dst, value);
      }
    }
  }

  /// Stores `value` into the atomic cell and returns the previous value.
  ///
  /// ```
  /// let ac = wtx::sync::AtomicCell::new(7);
  /// assert_eq!(ac.load(), 7);
  /// assert_eq!(ac.swap(8), 7);
  /// assert_eq!(ac.load(), 8);
  /// ```
  #[inline]
  pub fn swap(&self, value: T) -> T {
    let dst = self.value.get();
    let _guard = lock(dst.addr()).write();
    // SAFETY: pointer doesn't have offsets and `value` has the same size and alignment of `self`
    unsafe { ptr::replace(dst, value) }
  }

  fn as_ptr(&self) -> *mut T {
    self.value.get()
  }
}

impl<T> AtomicCell<T>
where
  T: Copy,
{
  /// Loads a value from the atomic cell.
  ///
  /// ```rust
  /// let ac = wtx::sync::AtomicCell::new(7);
  /// assert_eq!(ac.load(), 7);
  /// ```
  #[inline]
  pub fn load(&self) -> T {
    let src = self.as_ptr();
    let lock = lock(src.addr());

    if let Some(stamp) = lock.optimistic_read() {
      // SAFETY: pointer doesn't have offsets
      let value = unsafe { ptr::read_volatile(src) };
      if lock.validate_read(stamp) {
        return value;
      }
    }

    let guard = lock.write();
    // SAFETY: pointer comes from inner data and doesn't contains offsets
    let value = unsafe { ptr::read(src) };
    guard.abort();
    value
  }
}

impl<T> AtomicCell<T>
where
  T: Copy + Eq,
{
  /// If the current value equals `curr`, stores `new` into the atomic cell.
  ///
  /// The return value is a result indicating whether the new value was written and containing
  /// the previous value. On success this value is guaranteed to be equal to `curr`.
  ///
  /// ```
  /// let num = wtx::sync::AtomicCell::new(1);
  /// assert_eq!(num.compare_exchange(2, 3), Err(1));
  /// assert_eq!(num.load(), 1);
  /// assert_eq!(num.compare_exchange(1, 2), Ok(1));
  /// assert_eq!(num.load(), 2);
  /// ```
  #[inline]
  pub fn compare_exchange(&self, curr: T, new: T) -> Result<T, T> {
    let dest = self.as_ptr();
    let guard = lock(dest.addr()).write();
    // SAFETY: pointer comes from inner data and doesn't contains offsets
    if T::eq(unsafe { &*dest }, &curr) {
      // SAFETY: pointer comes from inner data and doesn't contains offsets
      Ok(unsafe { ptr::replace(dest, new) })
    } else {
      // SAFETY: pointer comes from inner data and doesn't contains offsets
      let elem = unsafe { ptr::read(dest) };
      guard.abort();
      Err(elem)
    }
  }

  /// Fetches the value, and applies a function to it that returns an optional
  /// new value. Returns a `Result` of `Ok(previous_value)` if the function returned `Some(_)`, else
  /// `Err(previous_value)`.
  ///
  /// ```rust
  /// let num = wtx::sync::AtomicCell::new(7);
  /// assert_eq!(num.fetch_update(|_| None), Err(7));
  /// assert_eq!(num.fetch_update(|local_num| Some(local_num + 1)), Ok(7));
  /// assert_eq!(num.fetch_update(|local_num| Some(local_num + 1)), Ok(8));
  /// assert_eq!(num.load(), 9);
  /// ```
  #[inline]
  pub fn fetch_update(&self, mut cb: impl FnMut(T) -> Option<T>) -> Result<T, T> {
    let mut prev = self.load();
    while let Some(next) = cb(prev) {
      match self.compare_exchange(prev, next) {
        elem @ Ok(_) => return elem,
        Err(next_prev) => prev = next_prev,
      }
    }
    Err(prev)
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

impl<T> Drop for AtomicCell<T> {
  #[inline]
  fn drop(&mut self) {
    if mem::needs_drop::<T>() {
      // SAFETY:
      // - mutable reference prevents concurrent access and ensures a valid pointer
      // - `ManuallyDrop` prevents double free
      unsafe {
        self.as_ptr().drop_in_place();
      }
    }
  }
}

impl<T> From<T> for AtomicCell<T> {
  #[inline]
  fn from(value: T) -> AtomicCell<T> {
    AtomicCell::new(value)
  }
}

impl<T> RefUnwindSafe for AtomicCell<T> {}

// SAFETY: concurrent access is manually managed
unsafe impl<T> Send for AtomicCell<T> where T: Send {}

// SAFETY: concurrent access is manually managed
unsafe impl<T: Send> Sync for AtomicCell<T> {}

impl<T> UnwindSafe for AtomicCell<T> where T: Send {}

fn lock(addr: usize) -> &'static SeqLock {
  #[allow(clippy::indexing_slicing, reason = "modulo result will always be in-bounds")]
  &LOCKS[addr % LEN].0
}

#[cfg(feature = "serde")]
mod serde {
  use crate::sync::AtomicCell;
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
