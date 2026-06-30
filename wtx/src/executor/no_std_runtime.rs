use crate::{executor::Runtime, sync::Backoff};
use core::{
  fmt::{Debug, Formatter},
  pin::pin,
  task::{Context, Poll, Waker},
};

/// A very limited `no_std` runtime that only spins until the future resolves.
#[derive(Clone, Copy, Default)]
pub struct NoStdRuntime {}

impl NoStdRuntime {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }
}

impl Runtime for NoStdRuntime {
  #[inline]
  fn new() -> crate::Result<Self> {
    Ok(Self::new())
  }

  #[inline]
  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future,
  {
    let mut pinned = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    let backoff = Backoff::new();
    loop {
      match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(el) => return el,
        Poll::Pending => backoff.snooze(),
      }
    }
  }
}

impl Debug for NoStdRuntime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("StdRuntime").finish()
  }
}
