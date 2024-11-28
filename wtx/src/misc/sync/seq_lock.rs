use crate::misc::{facades::atomic_usize::AtomicUsize, Backoff};
use core::{
  mem,
  sync::atomic::{fence, Ordering},
};

pub(crate) struct SeqLock {
  state: AtomicUsize,
}

impl SeqLock {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { state: AtomicUsize::new(0) }
  }

  #[inline]
  pub(crate) fn optimistic_read(&self) -> Option<usize> {
    let state = self.state.load(Ordering::Acquire);
    if state == 1 {
      None
    } else {
      Some(state)
    }
  }

  /// Returns `true` if the current stamp is equal to `stamp`.
  ///
  /// This method should be called after optimistic reads to check whether they are valid. The
  /// argument `stamp` should correspond to the one returned by method `optimistic_read`.
  #[inline]
  pub(crate) fn validate_read(&self, stamp: usize) -> bool {
    fence(Ordering::Acquire);
    self.state.load(Ordering::Relaxed) == stamp
  }

  #[inline]
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
  #[inline]
  pub(crate) fn abort(self) {
    self.lock.state.store(self.state, Ordering::Release);
    #[allow(clippy::mem_forget, reason = "avoids incrementing the stamp")]
    mem::forget(self);
  }
}

impl Drop for SeqLockWriteGuard {
  #[inline]
  fn drop(&mut self) {
    self.lock.state.store(self.state.wrapping_add(2), Ordering::Release);
  }
}
