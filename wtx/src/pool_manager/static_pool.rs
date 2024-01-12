use crate::{
  misc::PollOnce,
  pool_manager::{Lock, LockGuard, Pool, ResourceManager},
};
use alloc::collections::VecDeque;
use core::{
  array,
  cell::RefCell,
  pin::pin,
  sync::atomic::{AtomicUsize, Ordering},
};

/// A [StaticPool] synchronized by [RefCell].
pub type StaticPoolRefCell<R, RM, const N: usize> = StaticPool<RefCell<Option<R>>, RM, N>;
/// A [StaticPool] synchronized by [tokio::sync::Mutex].
#[cfg(feature = "tokio")]
pub type StaticPoolTokioMutex<R, RM, const N: usize> =
  StaticPool<tokio::sync::Mutex<Option<R>>, RM, N>;

/// Fixed-size pool that does not require the use of reference-counters. On the other hand, all
/// elements of the pool **must** be of type `Option<T>` to lazily evaluate resources.
///
/// If stack memory becomes a problem, try heap allocation.
#[derive(Debug)]
pub struct StaticPool<RL, RM, const N: usize> {
  idx: AtomicUsize,
  locks: [RL; N],
  rm: RM,
}

impl<R, RL, RM, const N: usize> StaticPool<RL, RM, N>
where
  RL: Lock<Option<R>>,
  RM: ResourceManager<Resource = R>,
{
  /// Sometimes it is desired to eagerly initialize all instances.
  pub async fn init_all(&self) -> Result<(), RM::Error> {
    for _ in 0..N {
      let _guard = self.get().await?;
    }
    Ok(())
  }
}

impl<R, RL, RM, const N: usize> Pool for StaticPool<RL, RM, N>
where
  RL: Lock<Option<R>>,
  RM: ResourceManager<Resource = R>,
{
  type Guard<'lock> = <RL::Guard<'lock> as LockGuard<'lock, Option<R>>>::Mapped<R>
  where
    <Self::ResourceManager as ResourceManager>::Resource: 'lock,
    Self: 'lock;

  type ResourceManager = RM;

  #[inline]
  fn new(rm: RM) -> crate::Result<Self> {
    if N == 0 {
      return Err(crate::Error::StaticPoolMustHaveCapacityForAtLeastOneElement);
    }
    let mut available = VecDeque::new();
    available.extend(0..N);
    Ok(Self { idx: AtomicUsize::new(0), locks: array::from_fn(|_| RL::new(None)), rm })
  }

  #[allow(
    // Inner code does not trigger `panic!`
    clippy::missing_panics_doc
  )]
  #[inline]
  async fn get<'this>(&'this self) -> Result<Self::Guard<'this>, RM::Error>
  where
    <Self::ResourceManager as ResourceManager>::Resource: 'this,
  {
    loop {
      #[allow(
        // `N` will never be zero
        clippy::arithmetic_side_effects,
      )]
      let local_idx = self.idx.fetch_add(1, Ordering::Release) % N;
      #[allow(
        // `locks` is an array that will always have a valid `self.idx % N` element
        clippy::unwrap_used
      )]
      let lock = self.locks.get(local_idx).unwrap();
      if let Some(mut guard) = PollOnce(pin!(lock.lock())).await {
        match &mut *guard {
          None => {
            *guard = Some(self.rm.create().await?);
          }
          Some(resource) if self.rm.is_invalid(resource) => {
            self.rm.recycle(resource).await?;
          }
          _ => {}
        }
        #[allow(
          // The above match took care of nullable guards
          clippy::unwrap_used
        )]
        return Ok(LockGuard::map(guard, |el| el.as_mut().unwrap()));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::pool_manager::{static_pool::StaticPoolRefCell, Pool, SimpleRM};

  #[tokio::test]
  async fn modifies_elements() {
    let pool = pool();
    assert_eq!([*pool.get().await.unwrap(), *pool.get().await.unwrap()], [0, 0]);
    *pool.get().await.unwrap() = 1;
    *pool.get().await.unwrap() = 2;
    assert_eq!([*pool.get().await.unwrap(), *pool.get().await.unwrap()], [1, 2]);
    *pool.get().await.unwrap() = 3;
    assert_eq!([*pool.get().await.unwrap(), *pool.get().await.unwrap()], [2, 3]);
  }

  #[tokio::test]
  async fn held_lock_is_not_modified() {
    let pool = pool();
    let lock = pool.get().await.unwrap();
    *pool.get().await.unwrap() = 1;
    assert_eq!([*lock, *pool.get().await.unwrap()], [0, 1]);
    *pool.get().await.unwrap() = 2;
    assert_eq!([*lock, *pool.get().await.unwrap()], [0, 2]);
    drop(lock);
    *pool.get().await.unwrap() = 1;
    assert_eq!([*pool.get().await.unwrap(), *pool.get().await.unwrap()], [2, 1]);
  }

  fn pool() -> StaticPoolRefCell<i32, SimpleRM<crate::Error, (), i32>, 2> {
    fn cb(_: &()) -> crate::Result<i32> {
      Ok(0)
    }
    StaticPoolRefCell::new(SimpleRM::new(cb, ())).unwrap()
  }
}
