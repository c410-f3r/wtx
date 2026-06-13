use crate::{executor::Runtime, sync::Backoff};
use core::{
  fmt::{Debug, Formatter},
  future::Future,
  pin::pin,
  task::{Context, Poll, Waker},
};

// A very limited `no_std` runtime that only spins until the future resolves.
#[derive(Clone, Copy, Default)]
pub struct NoStdRuntime {}

impl NoStdRuntime {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }

  /// Blocks the current execution context until the future resolves.
  pub fn block_on<F>(future: F) -> F::Output
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

impl Runtime for NoStdRuntime {
  fn optioned() -> crate::Result<Self> {
    Ok(Self::new())
  }

  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future,
  {
    Self::block_on(future)
  }
}

impl Debug for NoStdRuntime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("StdRuntime").finish()
  }
}
