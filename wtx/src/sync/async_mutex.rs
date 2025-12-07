use crate::{
  collection::{Deque, backward_deque_idx},
  misc::_unlikely_unreachable,
  sync::{AtomicUsize, SyncMutex, sync_mutex::SyncMutexGuard},
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
pub struct AsyncMutex<T> {
  state: AtomicUsize,
  value: UnsafeCell<T>,
  waiters: SyncMutex<Waiters>,
}

impl<T> AsyncMutex<T> {
  /// Creates a new futures-aware mutex.
  #[cfg(feature = "loom")]
  #[inline]
  pub fn new(t: T) -> Self {
    Self {
      state: AtomicUsize::new(0),
      value: UnsafeCell::new(t),
      waiters: SyncMutex::new(Waiters {
        added: 0,
        deque: Deque::new(),
        last_added: 0,
        waiting_count: 0,
      }),
    }
  }
  /// Creates a new futures-aware mutex.
  #[cfg(not(feature = "loom"))]
  #[inline]
  pub const fn new(t: T) -> Self {
    Self {
      state: AtomicUsize::new(0),
      value: UnsafeCell::new(t),
      waiters: SyncMutex::new(Waiters {
        added: 0,
        deque: Deque::new(),
        last_added: 0,
        waiting_count: 0,
      }),
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
  pub const fn lock(&self) -> AsyncMutexGuardFuture<'_, T> {
    AsyncMutexGuardFuture { idx_opt: None, mutex_opt: Some(self) }
  }

  /// Attempt to acquire the lock immediately.
  ///
  /// If the lock is currently held, returns `None`.
  #[inline]
  pub fn try_lock(&self) -> Option<AsyncMutexGuard<'_, T>> {
    let prev = self.state.fetch_or(IS_LOCKED, Ordering::Acquire);
    if is_locked(prev) { None } else { Some(AsyncMutexGuard { mutex: self }) }
  }
}

#[expect(clippy::missing_fields_in_debug, reason = "best effort")]
impl<T> Debug for AsyncMutex<T> {
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
unsafe impl<T: Send> Send for AsyncMutex<T> {}
// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Sync for AsyncMutex<T> {}

/// An RAII guard returned by the `lock` and `try_lock` methods. When this structure is dropped
/// (falls out of scope), the lock will be unlocked.
#[clippy::has_significant_drop]
pub struct AsyncMutexGuard<'any, T> {
  mutex: &'any AsyncMutex<T>,
}

impl<T> Debug for AsyncMutexGuard<'_, T>
where
  T: Debug,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("AsyncMutexGuard").field("mutex", &self.mutex).field("value", &&**self).finish()
  }
}

impl<T> Deref for AsyncMutexGuard<'_, T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &T {
    // SAFETY: We have exclusive access to the mutex.
    unsafe { &*self.mutex.value.get() }
  }
}

impl<T> DerefMut for AsyncMutexGuard<'_, T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut T {
    // SAFETY: We have exclusive access to the mutex.
    unsafe { &mut *self.mutex.value.get() }
  }
}

impl<T> Drop for AsyncMutexGuard<'_, T> {
  #[inline]
  fn drop(&mut self) {
    let prev = self.mutex.state.fetch_and(!IS_LOCKED, Ordering::AcqRel);
    if has_waiters(prev) {
      wake(&self.mutex.state, self.mutex.waiters.lock());
    }
  }
}

// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Send for AsyncMutexGuard<'_, T> {}
// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Sync> Sync for AsyncMutexGuard<'_, T> {}

/// A future which resolves when the target mutex has been successfully acquired.
#[derive(Debug)]
pub struct AsyncMutexGuardFuture<'mutex, T> {
  idx_opt: Option<usize>,
  // `None` indicates that the mutex was successfully acquired.
  mutex_opt: Option<&'mutex AsyncMutex<T>>,
}

impl<T> Drop for AsyncMutexGuardFuture<'_, T> {
  #[inline]
  fn drop(&mut self) {
    let (Some(idx), Some(mutex)) = (self.idx_opt, self.mutex_opt) else {
      return;
    };
    let mut guard = mutex.waiters.lock();
    if matches!(remove_waker(idx, &mutex.state, &mut guard), Some(Waiter::Woken)) {
      // Someone else awaked this instance while it is being dropped, which means that the `Drop`
      // implementation of `AsyncMutexGuard` will never call `wake`.
      wake(&mutex.state, guard);
    }
  }
}

impl<'any, T> Future for AsyncMutexGuardFuture<'any, T> {
  type Output = AsyncMutexGuard<'any, T>;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Some(mutex) = self.mutex_opt else { _unlikely_unreachable() };

    if let Some(mutex_guard) = mutex.try_lock() {
      if let Some(idx) = self.idx_opt {
        drop(remove_waker(idx, &mutex.state, &mut mutex_guard.mutex.waiters.lock()));
      }
      self.mutex_opt = None;
      return Poll::Ready(mutex_guard);
    }

    let AsyncMutex { state, waiters, value: _ } = mutex;
    let mut waiters_guard = waiters.lock();
    if let Some(idx) = self.idx_opt {
      let Waiters { added: _, deque, last_added, waiting_count } = &mut *waiters_guard;
      let actual_idx = backward_deque_idx(idx, *last_added);
      if let Some(elem) = deque.get_mut(actual_idx) {
        elem.register(waiting_count, cx.waker());
        if *waiting_count > 0 {
          let _ = state.fetch_or(HAS_WAITERS, Ordering::Relaxed);
        }
      }
    } else {
      waiters_guard.last_added = waiters_guard.added;
      self.idx_opt = Some(waiters_guard.last_added);
      waiters_guard.added = waiters_guard.added.wrapping_add(1);
      let _rslt = waiters_guard.deque.push_front(Waiter::Waiting(cx.waker().clone()));
      waiters_guard.waiting_count = waiters_guard.waiting_count.wrapping_add(1);
      if waiters_guard.waiting_count == 1 {
        let _ = state.fetch_or(HAS_WAITERS, Ordering::Relaxed);
      }
    }

    if let Some(mutex_guard) = mutex.try_lock() {
      if let Some(idx) = self.idx_opt {
        drop(remove_waker(idx, &mutex.state, &mut waiters_guard));
      }
      drop(waiters_guard);
      self.mutex_opt = None;
      return Poll::Ready(mutex_guard);
    }

    // Lock failed, waiter is registered. We return pending.
    Poll::Pending
  }
}

// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Send for AsyncMutexGuardFuture<'_, T> {}
// SAFETY: Access is exclusive regardless of the number of threads
unsafe impl<T: Send> Sync for AsyncMutexGuardFuture<'_, T> {}

#[derive(Debug)]
enum Waiter {
  Removed,
  Waiting(Waker),
  Woken,
}

impl Waiter {
  #[inline]
  fn register(&mut self, waiting_count: &mut usize, waker: &Waker) {
    match self {
      Self::Removed | Self::Woken => {
        *waiting_count = waiting_count.wrapping_add(1);
        *self = Self::Waiting(waker.clone());
      }
      Self::Waiting(elem) => {
        elem.clone_from(waker);
      }
    }
  }
}

#[derive(Debug)]
struct Waiters {
  added: usize,
  deque: Deque<Waiter>,
  last_added: usize,
  waiting_count: usize,
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
fn remove_waker(idx: usize, state: &AtomicUsize, waiters: &mut Waiters) -> Option<Waiter> {
  let actual_idx = backward_deque_idx(idx, waiters.last_added);
  let waiter = waiters.deque.get_mut(actual_idx)?;
  let prev = mem::replace(waiter, Waiter::Removed);
  if matches!(&prev, Waiter::Waiting(_)) {
    waiters.waiting_count = waiters.waiting_count.wrapping_sub(1);
    if waiters.waiting_count == 0 {
      let _ = state.fetch_and(!HAS_WAITERS, Ordering::Relaxed);
    }
  }
  Some(prev)
}

#[inline]
fn wake(state: &AtomicUsize, mut waiters: SyncMutexGuard<'_, Waiters>) {
  let waker_opt = loop {
    let Some(waiter) = waiters.deque.last_mut() else {
      let _ = state.fetch_and(!HAS_WAITERS, Ordering::Relaxed);
      break None;
    };
    let prev = mem::replace(waiter, Waiter::Woken);
    match prev {
      Waiter::Removed => {
        let _elem = waiters.deque.pop_back();
      }
      Waiter::Waiting(waker) => {
        waiters.waiting_count = waiters.waiting_count.wrapping_sub(1);
        if waiters.waiting_count == 0 {
          let _ = state.fetch_and(!HAS_WAITERS, Ordering::Relaxed);
        }
        break Some(waker);
      }
      Waiter::Woken => break None,
    }
  };
  drop(waiters);
  if let Some(waker) = waker_opt {
    waker.wake();
  }
}

#[cfg(all(feature = "loom", test))]
mod loom_tests {
  use crate::{
    collection::Vector,
    sync::{Arc, AsyncMutex},
  };

  #[test]
  fn addition() {
    const THREADS: usize = 4;

    loom::model(|| {
      let mutex = Arc::new(AsyncMutex::new(0));
      let vector = Vector::from_iterator((0..THREADS).map(|_| {
        let local_mutex = mutex.clone();
        loom::thread::spawn(move || {
          loom::future::block_on(async {
            let mut guard = local_mutex.lock().await;
            *guard += 1;
          });
        })
      }))
      .unwrap();
      for jh in vector {
        jh.join().unwrap();
      }
      loom::future::block_on(async {
        assert_eq!(*mutex.lock().await, THREADS);
      });
    });
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    executor::Runtime,
    misc::PollOnce,
    sync::{
      Arc, AsyncMutex,
      async_mutex::{has_waiters, is_locked},
    },
  };
  use core::sync::atomic::Ordering;

  #[test]
  fn competition() {
    let (tx, rx) = std::sync::mpsc::channel();
    let mutex = Arc::new(AsyncMutex::new(0));
    let num_threads = 1000;
    let runtime = Runtime::new();
    let tx = Arc::new(tx);

    for _ in 0..num_threads {
      let tx = tx.clone();
      let mutex = mutex.clone();
      let _fut = runtime
        .spawn_threaded(async move {
          let mut guard = mutex.lock().await;
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
        assert_eq!(num_threads, *mutex.lock().await);
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
        let mutex = AsyncMutex::new(());
        for _ in 0..10 {
          let _guard = mutex.lock().await;
        }
        check_mutex(&mutex);
      })
      .unwrap();
  }

  #[test]
  fn wakes_waiter() {
    Runtime::new()
      .block_on(async {
        let mutex = AsyncMutex::new(());
        {
          let lock0 = mutex.lock().await;
          let mut lock1_fut = mutex.lock();
          assert!(PollOnce::new(&mut lock1_fut).await.is_none());
          drop(lock0);
          assert!(PollOnce::new(&mut lock1_fut).await.is_some());
        }
        check_mutex(&mutex);
      })
      .unwrap();
  }

  fn check_mutex<T>(mutex: &AsyncMutex<T>) {
    let state = mutex.state.load(Ordering::Relaxed);
    let waiters = mutex.waiters.lock();
    assert_eq!(has_waiters(state), false);
    assert_eq!(is_locked(state), false);
    assert_eq!(waiters.waiting_count, 0);
  }
}
