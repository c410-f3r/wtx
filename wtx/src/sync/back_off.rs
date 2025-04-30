use core::{
  cell::Cell,
  fmt::{Debug, Formatter},
  hint::spin_loop,
};

const SPIN_LIMIT: u32 = 6;
const YIELD_LIMIT: u32 = 10;

/// Performs exponential backoff in spin loops, which reduces contention that may improve overall
/// performance.
pub struct Backoff {
  step: Cell<u32>,
}

impl Backoff {
  /// Creates a new instance.
  #[inline]
  pub const fn new() -> Self {
    Backoff { step: Cell::new(0) }
  }

  /// Returns `true` if exponential backoff has completed and blocking the thread is advised.
  #[inline]
  pub fn is_completed(&self) -> bool {
    self.step.get() > YIELD_LIMIT
  }

  /// Resets the instance.
  #[inline]
  pub fn reset(&self) {
    self.step.set(0);
  }

  /// Backs off in a blocking loop.
  ///
  /// Should be used when it is desirable to wait for another thread to make progress.
  #[inline]
  pub fn snooze(&self) {
    if self.step.get() <= SPIN_LIMIT {
      let until = 1 << self.step.get();
      for _ in 0..until {
        spin_loop();
      }
    } else {
      #[cfg(feature = "std")]
      std::thread::yield_now();
      #[cfg(not(feature = "std"))]
      {
        let until = 1 << self.step.get();
        for _ in 0..until {
          spin_loop();
        }
      }
    }
    if self.step.get() <= YIELD_LIMIT {
      self.step.set(self.step.get().wrapping_add(1));
    }
  }

  /// Backs off in a lock-free loop.
  ///
  /// Should be used when it is desirable to retry an operation because another thread made
  /// progress.
  #[inline]
  pub fn spin(&self) {
    let until = 1 << self.step.get().min(SPIN_LIMIT);
    for _ in 0..until {
      spin_loop();
    }
    if self.step.get() <= SPIN_LIMIT {
      self.step.set(self.step.get().wrapping_add(1));
    }
  }
}

impl Debug for Backoff {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Backoff")
      .field("is_completed", &self.is_completed())
      .field("step", &self.step)
      .finish()
  }
}

impl Default for Backoff {
  #[inline]
  fn default() -> Backoff {
    Backoff::new()
  }
}
