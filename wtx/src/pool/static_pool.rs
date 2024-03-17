use crate::{
  misc::{Lock, LockGuard, PollOnce, _unreachable},
  pool::{Pool, ResourceManager},
};
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
/// Useful when a more fine-grained control over resources is not necessary. If stack memory
/// becomes a problem, try heap allocation.
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
  for<'any> R: 'any,
{
  /// Sometimes it is desired to eagerly initialize all instances.
  pub async fn init_all<'this>(
    &'this self,
    ca: &RM::CreateAux,
    ra: &RM::RecycleAux,
  ) -> Result<(), RM::Error> {
    for _ in 0..N {
      let _guard = self.get(ca, ra).await?;
    }
    Ok(())
  }
}

impl<R, RL, RM, const N: usize> Pool for StaticPool<RL, RM, N>
where
  RL: Lock<Option<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> R: 'any,
{
  type Guard<'lock> = <RL::Guard<'lock> as LockGuard<'lock, Option<R>>>::Mapped<R>
  where
    Self: 'lock;
  type ResourceManager = RM;

  #[inline]
  fn new(rm: RM) -> Self {
    if N == 0 {
      panic!("Static pools need to contain at least one element");
    }
    Self { idx: AtomicUsize::new(0), locks: array::from_fn(|_| RL::new(None)), rm }
  }

  #[inline]
  async fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> Result<Self::Guard<'_>, RM::Error> {
    loop {
      #[allow(
        // `N` will never be zero
        clippy::arithmetic_side_effects,
      )]
      let local_idx = self.idx.fetch_add(1, Ordering::Release) % N;
      let lock = match self.locks.get(local_idx) {
        Some(elem) => elem,
        None => _unreachable(),
      };
      if let Some(mut guard) = PollOnce(pin!(lock.lock())).await {
        match &mut *guard {
          None => {
            *guard = Some(self.rm.create(ca).await?);
          }
          Some(resource) if self.rm.is_invalid(resource) => {
            self.rm.recycle(ra, resource).await?;
          }
          _ => {}
        }
        return Ok(LockGuard::map(guard, |el| match el.as_mut() {
          Some(elem) => elem,
          None => _unreachable(),
        }));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::pool::{static_pool::StaticPoolRefCell, Pool, SimpleRM};

  #[tokio::test]
  async fn modifies_elements() {
    let pool = pool();
    assert_eq!([*pool.get(&(), &()).await.unwrap(), *pool.get(&(), &()).await.unwrap()], [0, 0]);
    *pool.get(&(), &()).await.unwrap() = 1;
    *pool.get(&(), &()).await.unwrap() = 2;
    assert_eq!([*pool.get(&(), &()).await.unwrap(), *pool.get(&(), &()).await.unwrap()], [1, 2]);
    *pool.get(&(), &()).await.unwrap() = 3;
    assert_eq!([*pool.get(&(), &()).await.unwrap(), *pool.get(&(), &()).await.unwrap()], [2, 3]);
  }

  #[tokio::test]
  async fn held_lock_is_not_modified() {
    let pool = pool();
    let lock = pool.get(&(), &()).await.unwrap();
    *pool.get(&(), &()).await.unwrap() = 1;
    assert_eq!([*lock, *pool.get(&(), &()).await.unwrap()], [0, 1]);
    *pool.get(&(), &()).await.unwrap() = 2;
    assert_eq!([*lock, *pool.get(&(), &()).await.unwrap()], [0, 2]);
    drop(lock);
    *pool.get(&(), &()).await.unwrap() = 1;
    assert_eq!([*pool.get(&(), &()).await.unwrap(), *pool.get(&(), &()).await.unwrap()], [2, 1]);
  }

  fn pool() -> StaticPoolRefCell<i32, SimpleRM<crate::Error, (), i32>, 2> {
    fn cb(_: &()) -> crate::Result<i32> {
      Ok(0)
    }
    StaticPoolRefCell::new(SimpleRM::new(cb, ()))
  }
}
