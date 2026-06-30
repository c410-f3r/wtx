use crate::{
  executor::{ExecutorError, Runtime},
  sync::{Arc, AtomicWaker},
};
use alloc::task::Wake;
use core::{
  cell::RefCell,
  fmt::{Debug, Formatter},
  pin::{Pin, pin},
  task::{Context, Poll, Waker},
};
use std::{sync::mpsc::Receiver, thread, thread_local};

thread_local! {
  static BLOCK_ON: RefCell<Waker> = RefCell::new(CurrThreadWaker::waker());
}

/// Simple dependency-free runtime intended for tests, toy programs and demonstrations.
#[derive(Clone, Copy)]
pub struct StdRuntime {}

impl StdRuntime {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }

  /// Blocks the current thread on a future.
  #[inline]
  pub fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future,
  {
    let pinned_future = pin!(future);
    BLOCK_ON.with(|cache| {
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
    })
  }

  /// Spawns a new asynchronous task.
  #[inline]
  pub fn spawn<F>(&self, future: F) -> crate::Result<SpawnFuture<F::Output>>
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
    Ok(SpawnFuture { atomic_waker, receiver })
  }

  /// Spawns a `!Send` future on the current thread.
  #[inline]
  pub fn spawn_local<F>(&self, _: F) -> crate::Result<SpawnFuture<F::Output>>
  where
    F: Future + 'static,
  {
    Err(ExecutorError::UnsupportedStdSpawnLocal.into())
  }
}

impl Runtime for StdRuntime {
  #[inline]
  fn new() -> crate::Result<Self> {
    Ok(Self::new())
  }

  #[inline]
  fn block_on<F>(&self, future: F) -> F::Output
  where
    F: Future,
  {
    (*self).block_on(future)
  }
}

impl Debug for StdRuntime {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("StdRuntime").finish()
  }
}

impl Default for StdRuntime {
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

/// Returned by [`StdRuntime::spawn`]
#[derive(Debug)]
pub struct SpawnFuture<T> {
  atomic_waker: Arc<AtomicWaker>,
  receiver: Receiver<T>,
}

impl<T> Future for SpawnFuture<T> {
  type Output = T;

  #[inline]
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.atomic_waker.register(cx.waker());
    // If `Err`, then the thread didn't initiate but the waker is registered for posterior calls.
    // If `Ok`, then the thread initialized with or withouts `wake` calls.
    if let Ok(elem) = self.receiver.try_recv() {
      return Poll::Ready(elem);
    }
    Poll::Pending
  }
}

struct CurrThreadWaker {
  thread: thread::Thread,
}

impl CurrThreadWaker {
  fn waker() -> Waker {
    Waker::from(alloc::sync::Arc::new(CurrThreadWaker { thread: thread::current() }))
  }
}

impl Wake for CurrThreadWaker {
  #[inline]
  fn wake(self: alloc::sync::Arc<Self>) {
    self.thread.unpark();
  }

  #[inline]
  fn wake_by_ref(self: &alloc::sync::Arc<Self>) {
    self.thread.unpark();
  }
}
