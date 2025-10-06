use crate::{
  collection::{Deque, backward_deque_idx},
  sync::AtomicUsize,
};
use core::{
  cell::UnsafeCell,
  fmt::{Debug, Formatter},
  mem,
  ops::{Deref, DerefMut},
  pin::Pin,
  sync::atomic::Ordering,
  task::{Context, Poll, Waker},
};

const HAS_WAITERS: usize = 0b10;
const IS_LOCKED: usize = 0b1;

/// An asynchronous `Mutex`-like type.
pub struct Mutex<T> {
  state: AtomicUsize,
  waiters: std::sync::Mutex<Waiters>,
  value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
  /// Creates a new futures-aware mutex.
  #[inline]
  pub const fn new(t: T) -> Self {
    Self {
      state: AtomicUsize::new(0),
      waiters: std::sync::Mutex::new(Waiters { added: 0, deque: Deque::new(), last_added: 0 }),
      value: UnsafeCell::new(t),
    }
  }

  /// A mutable reference ensures unique access, as such, it is safe to return it.
  #[inline]
  pub fn get_mut(&mut self) -> &mut T {
    // SAFETY: We have exclusive access to the mutex.
    unsafe { &mut *self.value.get() }
  }

  /// Consumes this mutex, returning the underlying data.
  #[inline]
  pub fn into_inner(self) -> T {
    self.value.into_inner()
  }

  /// Acquire the lock asynchronously.
  #[inline]
  pub const fn lock(&self) -> MutexLockFuture<'_, T> {
    MutexLockFuture { idx_opt: None, mutex_opt: Some(self) }
  }

  /// Attempt to acquire the lock immediately.
  ///
  /// If the lock is currently held, returns `None`.
  #[inline]
  pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
    let prev = self.state.fetch_or(IS_LOCKED, Ordering::Acquire);
    if is_locked(prev) { None } else { Some(MutexGuard { mutex: self }) }
  }
}

#[expect(clippy::missing_fields_in_debug, reason = "best effort")]
impl<T> Debug for Mutex<T> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    let state = self.state.load(Ordering::SeqCst);
    f.debug_struct("Mutex")
      .field("is_locked", &is_locked(state))
      .field("has_waiters", &has_waiters(state))
      .finish()
  }
}

// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Send for Mutex<T> {}
// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send + Sync> Sync for Mutex<T> {}

/// An RAII guard returned by the `lock` and `try_lock` methods. When this structure is dropped
/// (falls out of scope), the lock will be unlocked.
#[clippy::has_significant_drop]
pub struct MutexGuard<'any, T> {
  mutex: &'any Mutex<T>,
}

impl<T> Debug for MutexGuard<'_, T>
where
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("MutexGuard").field("mutex", &self.mutex).field("value", &&**self).finish()
  }
}

impl<T> Deref for MutexGuard<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    // SAFETY: We have exclusive access to the mutex.
    unsafe { &*self.mutex.value.get() }
  }
}

impl<T> DerefMut for MutexGuard<'_, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    // SAFETY: We have exclusive access to the mutex.
    unsafe { &mut *self.mutex.value.get() }
  }
}

impl<T> Drop for MutexGuard<'_, T> {
  #[expect(clippy::unwrap_used, reason = "blame the std")]
  #[inline]
  fn drop(&mut self) {
    let prev = self.mutex.state.fetch_and(!IS_LOCKED, Ordering::AcqRel);
    if has_waiters(prev) {
      wake(&self.mutex.state, &mut self.mutex.waiters.lock().unwrap());
    }
  }
}

// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Send for MutexGuard<'_, T> {}
// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send + Sync> Sync for MutexGuard<'_, T> {}

/// A future which resolves when the target mutex has been successfully acquired.
#[derive(Debug)]
pub struct MutexLockFuture<'mutex, T> {
  idx_opt: Option<usize>,
  // `None` indicates that the mutex was successfully acquired.
  mutex_opt: Option<&'mutex Mutex<T>>,
}

impl<T> Drop for MutexLockFuture<'_, T> {
  #[expect(clippy::unwrap_used, reason = "blame the std")]
  #[inline]
  fn drop(&mut self) {
    let (Some(idx), Some(mutex)) = (self.idx_opt, self.mutex_opt) else {
      return;
    };
    let mut guard = mutex.waiters.lock().unwrap();
    if matches!(remove_waker(idx, &mut guard), Some(Waiter::Woken)) {
      // Someone else awaked this instance while it is being dropped, which means that the `Drop`
      // implementation of `MutexGuard` will never call `wake`.
      wake(&mutex.state, &mut guard);
    }
  }
}

impl<'any, T> Future for MutexLockFuture<'any, T> {
  type Output = crate::Result<MutexGuard<'any, T>>;

  #[expect(clippy::unwrap_used, reason = "blame the std")]
  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mutex = self.mutex_opt.ok_or(crate::Error::FuturePolledAfterFinalization)?;

    if let Some(guard) = mutex.try_lock() {
      if let Some(idx) = self.idx_opt {
        let _waiter = remove_waker(idx, &mut guard.mutex.waiters.lock().unwrap());
      }
      self.mutex_opt = None;
      return Poll::Ready(Ok(guard));
    }

    {
      let Mutex { state, waiters, value: _ } = mutex;
      let mut guard = waiters.lock().unwrap();
      if let Some(idx) = self.idx_opt {
        let actual_idx = backward_deque_idx(idx, guard.last_added);
        if let Some(elem) = guard.deque.get_mut(actual_idx) {
          elem.register(cx.waker());
        }
      } else {
        guard.last_added = guard.added;
        self.idx_opt = Some(guard.last_added);
        guard.added = guard.added.wrapping_add(1);
        let _rslt = guard.deque.push_front(Waiter::Waiting(cx.waker().clone()));
        if guard.deque.len() == 1 {
          let _ = state.fetch_or(HAS_WAITERS, Ordering::Relaxed);
        }
      }
    }

    if let Some(guard) = mutex.try_lock() {
      if let Some(idx) = self.idx_opt {
        let _waiter = remove_waker(idx, &mut guard.mutex.waiters.lock().unwrap());
      }
      self.mutex_opt = None;
      return Poll::Ready(Ok(guard));
    }

    Poll::Pending
  }
}

// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Send for MutexLockFuture<'_, T> {}
// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T> Sync for MutexLockFuture<'_, T> {}

#[derive(Debug)]
enum Waiter {
  Removed,
  Waiting(Waker),
  Woken,
}

impl Waiter {
  #[inline]
  fn register(&mut self, waker: &Waker) {
    match self {
      Self::Waiting(elem) if waker.will_wake(elem) => {}
      _ => *self = Self::Waiting(waker.clone()),
    }
  }
}

#[derive(Debug)]
struct Waiters {
  added: usize,
  deque: Deque<Waiter>,
  last_added: usize,
}

#[inline]
const fn is_locked(state: usize) -> bool {
  (state & IS_LOCKED) != 0
}

#[inline]
const fn has_waiters(state: usize) -> bool {
  (state & HAS_WAITERS) != 0
}

#[inline]
fn remove_waker(idx: usize, waiters: &mut Waiters) -> Option<Waiter> {
  let actual_idx = backward_deque_idx(idx, waiters.last_added);
  let waiter = waiters.deque.get_mut(actual_idx)?;
  let prev = mem::replace(waiter, Waiter::Removed);
  Some(prev)
}

#[inline]
fn wake(state: &AtomicUsize, waiters: &mut Waiters) {
  loop {
    let Some(waiter) = waiters.deque.last_mut() else {
      let _ = state.fetch_and(!HAS_WAITERS, Ordering::Relaxed);
      break;
    };
    let prev = mem::replace(waiter, Waiter::Woken);
    match prev {
      Waiter::Removed => {
        let _elem = waiters.deque.pop_back();
        continue;
      }
      Waiter::Waiting(waker) => waker.wake(),
      Waiter::Woken => {}
    }
    break;
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    executor::Runtime,
    misc::PollOnce,
    sync::{
      Arc, Mutex,
      mutex::{has_waiters, is_locked},
    },
  };
  use core::sync::atomic::Ordering;

  #[test]
  fn competition() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mutex = Arc::new(Mutex::new(0));
    let num_threads = 1000;
    let runtime = Runtime::new();
    let tx = Arc::new(tx);

    for _ in 0..num_threads {
      let tx = tx.clone();
      let mutex = mutex.clone();
      let _fut = runtime
        .spawn_threaded(async move {
          let mut guard = mutex.lock().await.unwrap();
          *guard += 1;
          tx.send(()).unwrap();
        })
        .unwrap();
    }

    runtime
      .block_on(async {
        for _ in 0..num_threads {
          rx.recv().unwrap();
        }
        assert_eq!(num_threads, *mutex.lock().await.unwrap());
      })
      .unwrap();

    // FIXME(MIRI): https://github.com/rust-lang/miri/issues/1371
    std::thread::sleep(std::time::Duration::from_millis(500));

    check_mutex(&mutex);
  }

  #[test]
  fn sequential() {
    Runtime::new()
      .block_on(async {
        let mutex = Mutex::new(());
        for _ in 0..10 {
          let _guard = mutex.lock().await.unwrap();
        }
        check_mutex(&mutex);
      })
      .unwrap();
  }

  #[test]
  fn wakes_waiter() {
    Runtime::new()
      .block_on(async {
        let mutex = Mutex::new(());
        {
          let lock0 = mutex.lock().await.unwrap();
          let mut lock1_fut = mutex.lock();
          assert!(PollOnce::new(&mut lock1_fut).await.is_none());
          drop(lock0);
          assert!(PollOnce::new(&mut lock1_fut).await.is_some());
        }
        check_mutex(&mutex);
      })
      .unwrap();
  }

  fn check_mutex<T>(mutex: &Mutex<T>) {
    let state = mutex.state.load(Ordering::Relaxed);
    let waiters = mutex.waiters.lock().unwrap();
    assert_eq!(has_waiters(state), false);
    assert_eq!(is_locked(state), false);
    assert_eq!(waiters.deque.len(), 0);
  }
}
