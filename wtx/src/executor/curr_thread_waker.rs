use alloc::{sync::Arc, task::Wake};
use core::task::Waker;
use std::thread;

pub(crate) struct CurrThreadWaker {
  pub(crate) thread: thread::Thread,
}

impl CurrThreadWaker {
  pub(crate) fn waker() -> Waker {
    Waker::from(Arc::new(CurrThreadWaker { thread: thread::current() }))
  }
}

impl Wake for CurrThreadWaker {
  #[inline]
  fn wake(self: Arc<Self>) {
    self.thread.unpark();
  }

  #[inline]
  fn wake_by_ref(self: &Arc<Self>) {
    self.thread.unpark();
  }
}
