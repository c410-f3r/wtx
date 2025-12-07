use crate::{executor::curr_thread_waker::CurrThreadWaker, sync::AtomicWaker};
use alloc::sync::Arc;
use core::{
  cell::RefCell,
  fmt::{Debug, Formatter},
  future::poll_fn,
  pin::{Pin, pin},
  task::{Context, Poll, Waker},
};
use std::{thread, thread_local};

/// Simple dependency-free runtime intended for tests, toy programs and demonstrations.
#[derive(Clone)]
pub struct Runtime(());

impl Runtime {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Runtime(())
  }

  /// Blocks the current thread on a future.
  #[inline]
  pub fn block_on<F>(&self, future: F) -> crate::Result<F::Output>
  where
    F: Future,
  {
    thread_local! {
      static CACHE: RefCell<Waker> = RefCell::new(CurrThreadWaker::waker());
    }
    let pinned_future = pin!(future);
    Ok(CACHE.with(|cache| {
      let new;
      let stored;
      let waker = if let Ok(elem) = cache.try_borrow_mut() {
        stored = elem;
        &*stored
      } else {
        new = CurrThreadWaker::waker();
        &new
      };
      work(Context::from_waker(waker), pinned_future)
    }))
  }

  /// Spawns a new thread in the background that will awake the returned future once finished.
  #[inline]
  pub fn spawn_threaded<F>(&self, future: F) -> crate::Result<impl Future<Output = F::Output>>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let atomic_waker = Arc::new(AtomicWaker::new());
    let atomic_waker_thread = Arc::clone(&atomic_waker);
    let _jh = thread::Builder::new().spawn(move || {
      let output = {
        let pinned_future = pin!(future);
        let local_waker = CurrThreadWaker::waker();
        work(Context::from_waker(&local_waker), pinned_future)
      };
      let _rslt = sender.send(output);
      atomic_waker_thread.wake();
    })?;
    Ok(poll_fn(move |cx| {
      atomic_waker.register(cx.waker());
      if let Ok(elem) = receiver.try_recv() {
        return Poll::Ready(elem);
      }
      Poll::Pending
    }))
  }
}

impl Debug for Runtime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Runtime").finish()
  }
}

impl Default for Runtime {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

fn work<F>(mut cx: Context<'_>, mut fut: Pin<&mut F>) -> F::Output
where
  F: Future,
{
  loop {
    match fut.as_mut().poll(&mut cx) {
      Poll::Ready(output) => break output,
      Poll::Pending => thread::park(),
    }
  }
}
