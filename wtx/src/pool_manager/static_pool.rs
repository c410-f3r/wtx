use crate::{
  misc::PollOnce,
  pool_manager::{Lock, LockGuard, ResourceManager},
};
use core::{
  array,
  pin::pin,
  sync::atomic::{AtomicUsize, Ordering},
};

/// Fixed-size pool that does not require the use of reference-counters. On the other side, all
/// elements of the pool **must** be of type `Option<T>` to lazily evaluate resources.
///
/// If stack memory becomes a problem, try heap allocation.
#[derive(Debug)]
pub struct StaticPool<L, M, const N: usize> {
  idx: AtomicUsize,
  locks: [L; N],
  rm: M,
}

impl<L, M, R, const N: usize> StaticPool<L, M, N>
where
  L: Lock<Option<R>>,
  M: ResourceManager<Resource = R>,
{
  /// Initializes inner elements.
  #[inline]
  pub fn new(rm: M) -> crate::Result<Self> {
    if N == 0 {
      return Err(crate::Error::StaticPoolMustHaveCapacityForAtLeastOneElement);
    }
    Ok(Self { idx: AtomicUsize::new(0), locks: array::from_fn(|_| L::new(None)), rm })
  }

  /// Tries to retrieve a free resource.
  ///
  /// If the resource does not exist, a new one is created and if the pool is full, this method will
  /// await until a free resource is available.
  #[allow(
    // Inner code does not trigger `panic!`
    clippy::missing_panics_doc
  )]
  #[inline]
  pub async fn get(
    &self,
  ) -> Result<<L::Guard<'_> as LockGuard<'_, Option<R>>>::Mapped<R>, M::Error> {
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
          Some(resource) => {
            if let Some(persistent) = self.rm.check_integrity(resource) {
              self.rm.recycle(persistent, resource).await?;
            }
          }
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
  use crate::pool_manager::{ResourceManager, StaticPool};
  use core::cell::RefCell;

  struct TestManager;

  impl ResourceManager for TestManager {
    type Persistent = ();
    type Error = crate::Error;
    type Resource = i32;

    #[inline]
    async fn create(&self) -> Result<Self::Resource, Self::Error> {
      Ok(0)
    }

    #[inline]
    fn check_integrity(&self, _: &mut Self::Resource) -> Option<Self::Persistent> {
      None
    }

    #[inline]
    async fn recycle(
      &self,
      _: Self::Persistent,
      _: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      Ok(())
    }
  }

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

  fn pool() -> StaticPool<RefCell<Option<i32>>, TestManager, 2> {
    StaticPool::new(TestManager).unwrap()
  }
}
