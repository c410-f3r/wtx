use crate::sync::{AtomicUsize, Backoff, Ordering, fence};
use core::mem;

/// Sequential lock
#[derive(Debug)]
pub struct SeqLock {
  state: AtomicUsize,
}

impl SeqLock {
  pub(crate) const fn new() -> Self {
    Self { state: AtomicUsize::new(0) }
  }

  pub(crate) fn optimistic_read(&self) -> Option<usize> {
    let state = self.state.load(Ordering::Acquire);
    if state == 1 { None } else { Some(state) }
  }

  /// Returns `true` if the current stamp is equal to `stamp`.
  ///
  /// This method should be called after optimistic reads to check whether they are valid. The
  /// argument `stamp` should correspond to the one returned by method `optimistic_read`.
  pub(crate) fn validate_read(&self, stamp: usize) -> bool {
    fence(Ordering::Acquire);
    self.state.load(Ordering::Relaxed) == stamp
  }

  pub(crate) fn write(&'static self) -> SeqLockWriteGuard {
    let backoff = Backoff::new();
    loop {
      let previous = self.state.swap(1, Ordering::Acquire);
      if previous != 1 {
        fence(Ordering::Release);
        return SeqLockWriteGuard { lock: self, state: previous };
      }
      backoff.snooze();
    }
  }
}

pub(crate) struct SeqLockWriteGuard {
  lock: &'static SeqLock,
  state: usize,
}

impl SeqLockWriteGuard {
  pub(crate) fn abort(self) {
    self.lock.state.store(self.state, Ordering::Release);
    #[allow(clippy::mem_forget, reason = "avoids incrementing the stamp")]
    mem::forget(self);
  }
}

impl Drop for SeqLockWriteGuard {
  fn drop(&mut self) {
    self.lock.state.store(self.state.wrapping_add(2), Ordering::Release);
  }
}
