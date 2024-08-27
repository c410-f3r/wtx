use core::{
  cell::UnsafeCell,
  fmt::{self, Debug},
  sync::atomic::{
    AtomicUsize,
    Ordering::{AcqRel, Acquire, Release},
  },
  task::Waker,
};

const WAITING: usize = 0;
const REGISTERING: usize = 0b01;
const WAKING: usize = 0b10;

/// [Waker] that can be shared across tasks.
pub struct AtomicWaker {
  state: AtomicUsize,
  waker: UnsafeCell<Option<Waker>>,
}

impl AtomicWaker {
  /// Creates an empty instance.
  #[inline]
  pub const fn new() -> Self {
    AtomicWaker { state: AtomicUsize::new(WAITING), waker: UnsafeCell::new(None) }
  }

  /// Registers the waker to be notified on calls to `wake`.
  #[inline]
  pub fn register(&self, waker: &Waker) {
    match self
      .state
      .compare_exchange(WAITING, REGISTERING, Acquire, Acquire)
      .unwrap_or_else(|el| el)
    {
      WAITING => {
        // SAFETY: `compare_exchange` manages concurrent accesses.
        let waker_opt = unsafe { &mut *self.waker.get() };
        match waker_opt {
          Some(elem) => elem.clone_from(waker),
          _ => *waker_opt = Some(waker.clone()),
        }
        if self.state.compare_exchange(REGISTERING, WAITING, AcqRel, Acquire).is_err() {
          let Some(local_waker) = waker_opt.take() else {
            return;
          };
          let _ = self.state.swap(WAITING, AcqRel);
          local_waker.wake();
        }
      }
      WAKING => {
        waker.wake_by_ref();
      }
      _ => {}
    }
  }

  /// Returns the last [Waker] passed to [`Self::register`], if any.
  #[inline]
  pub fn take(&self) -> Option<Waker> {
    match self.state.fetch_or(WAKING, AcqRel) {
      WAITING => {
        // SAFETY: Lock was acquire through `fetch_or` so the last waker can be retrieved.
        let waker = unsafe { (*self.waker.get()).take() };
        let _ = self.state.fetch_and(!WAKING, Release);
        waker
      }
      _ => None,
    }
  }

  /// Consumes the last [Waker] passed in [`Self::register`], if any.
  #[inline]
  pub fn wake(&self) {
    if let Some(waker) = self.take() {
      waker.wake();
    }
  }
}

impl Debug for AtomicWaker {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "AtomicWaker")
  }
}

impl Default for AtomicWaker {
  #[inline]
  fn default() -> Self {
    AtomicWaker::new()
  }
}

// SAFETY: Concurrent access is manually managed
unsafe impl Send for AtomicWaker {}

// SAFETY: Concurrent access is manually managed
unsafe impl Sync for AtomicWaker {}

#[cfg(test)]
mod tests {
  use crate::misc::AtomicWaker;
  use alloc::sync::Arc;
  use core::{
    future::poll_fn,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
  };
  use std::thread;
  use tokio::runtime::Builder;

  #[test]
  fn non_blocking_operation() {
    let atomic_waker = Arc::new(AtomicWaker::new());
    let atomic_waker_clone = atomic_waker.clone();

    let waiting = Arc::new(AtomicBool::new(false));
    let waiting_clone = waiting.clone();

    let woken = Arc::new(AtomicBool::new(false));
    let woken_clone = woken.clone();

    let jh = thread::spawn(move || {
      let mut pending = 0;
      Builder::new_current_thread().build().unwrap().block_on(poll_fn(move |cx| {
        if woken_clone.load(Ordering::Relaxed) {
          Poll::Ready(())
        } else {
          assert_eq!(0, pending);
          pending += 1;
          atomic_waker_clone.register(cx.waker());
          waiting_clone.store(true, Ordering::Relaxed);
          Poll::Pending
        }
      }))
    });

    while !waiting.load(Ordering::Relaxed) {}

    thread::yield_now();
    woken.store(true, Ordering::Relaxed);
    atomic_waker.wake();
    jh.join().unwrap();
  }
}
