use crate::{
  misc::{Lock, LockGuard, Queue, RefCounter, _unreachable},
  pool::{Pool, ResourceManager},
};
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

/// A [FixedPool] synchronized by [tokio::sync::Mutex].
#[cfg(feature = "tokio")]
pub type FixedPoolTokio<R, RM> =
  FixedPool<alloc::sync::Arc<tokio::sync::Mutex<Queue<usize>>>, tokio::sync::Mutex<Option<R>>, RM>;
/// A [FixedPoolGetRslt] synchronized by [tokio::sync::MappedMutexGuard].
#[cfg(feature = "tokio")]
pub type FixedPoolGetRsltTokio<'guard, R> = FixedPoolGetRslt<
  alloc::sync::Arc<tokio::sync::Mutex<Queue<usize>>>,
  tokio::sync::MappedMutexGuard<'guard, R>,
>;

/// A pool that does not dynamically change its size after initialization. All elements **must** be
/// of type `Option<T>` to lazily evaluate resources.
///
/// All `get` calls must end with a [FixedPoolController::release] invocation because otherwise
/// the underlying resource will be locked forever.
#[derive(Debug)]
pub struct FixedPool<IC, RL, RM> {
  indcs: IC,
  locks: Vec<RL>,
  rm: RM,
}

impl<IC, R, RL, RM> FixedPool<IC, RL, RM>
where
  IC: RefCounter + Lock<Resource = Queue<usize>>,
  IC::Item: Lock<Resource = Queue<usize>>,
  RL: Lock<Resource = Option<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> IC: 'any,
  for<'any> R: 'any,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  /// Creates a new instance with a fixed amount of memory delimited by `len`.
  ///
  /// If `0` is received, then `len` will be stored as `1`.
  #[inline]
  pub fn new(mut len: usize, rm: RM) -> Self {
    len = len.max(1);
    Self {
      indcs: {
        let mut rslt = Queue::with_capacity(len);
        for idx in 0..len {
          let _rslt = rslt.push_front(idx);
        }
        <IC as RefCounter>::new(IC::Item::new(rslt))
      },
      locks: {
        let mut rslt = Vec::with_capacity(len);
        rslt.extend((0..len).map(|_| RL::new(None)));
        rslt
      },
      rm,
    }
  }

  /// Sometimes it is desired to eagerly initialize all instances.
  #[inline]
  pub async fn init_all<'this>(
    &'this self,
    ca: &RM::CreateAux,
    ra: &RM::RecycleAux,
  ) -> Result<(), RM::Error> {
    for _ in 0..self.locks.len() {
      let _guard = self.get(ca, ra).await?;
    }
    Ok(())
  }
}

impl<IC, R, RL, RM> Pool for FixedPool<IC, RL, RM>
where
  IC: RefCounter + Lock<Resource = Queue<usize>>,
  RL: Lock<Resource = Option<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> IC: 'any,
  for<'any> R: 'any,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  type GetRslt<'guard> = FixedPoolGetRslt<IC, Self::Guard<'guard>>;
  type Guard<'guard> = <RL::Guard<'guard> as LockGuard<'guard, Option<R>>>::Mapped<R>;
  type GuardElement = R;
  type ResourceManager = RM;

  #[inline]
  async fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> Result<Self::GetRslt<'_>, RM::Error> {
    loop {
      let mut indcs_lock = self.indcs.lock().await;
      let Some(idx) = indcs_lock.pop_back() else {
        drop(indcs_lock);
        continue;
      };
      drop(indcs_lock);
      let lock = self.locks.get(idx).unwrap();
      let mut guard = lock.lock().await;
      match &mut *guard {
        None => {
          *guard = Some(self.rm.create(ca).await?);
        }
        Some(resource) if self.rm.is_invalid(resource) => {
          self.rm.recycle(ra, resource).await?;
        }
        _ => {}
      }
      let lock_guard = LockGuard::map(guard, |el| match el.as_mut() {
        Some(elem) => elem,
        None => _unreachable(),
      });
      return Ok(FixedPoolGetRslt { idx, indcs: self.indcs.clone(), lock_guard });
    }
  }
}

/// Controls the guard locks related to [FixedPool].
#[derive(Debug, PartialEq)]
pub struct FixedPoolGetRslt<IL, LG> {
  idx: usize,
  indcs: IL,
  lock_guard: LG,
}

impl<IL, LG> FixedPoolGetRslt<IL, LG>
where
  IL: Lock<Resource = Queue<usize>>,
{
  /// Releases the inner lock.
  #[inline]
  pub async fn release(self) -> LG {
    let _rslt = self.indcs.lock().await.push_front(self.idx);
    self.lock_guard
  }
}

impl<IL, LG> Deref for FixedPoolGetRslt<IL, LG>
where
  IL: Lock<Resource = Queue<usize>>,
{
  type Target = LG;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.lock_guard
  }
}

impl<IL, LG> DerefMut for FixedPoolGetRslt<IL, LG>
where
  IL: Lock<Resource = Queue<usize>>,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.lock_guard
  }
}

#[cfg(feature = "tokio")]
mod _tokio {
  use crate::{
    misc::{Lease, LeaseMut},
    pool::FixedPoolGetRslt,
  };
  use tokio::sync::MappedMutexGuard;

  impl<IL, R> Lease<R> for FixedPoolGetRslt<IL, MappedMutexGuard<'_, R>> {
    #[inline]
    fn lease(&self) -> &R {
      &self.lock_guard
    }
  }

  impl<IL, R> LeaseMut<R> for FixedPoolGetRslt<IL, MappedMutexGuard<'_, R>> {
    #[inline]
    fn lease_mut(&mut self) -> &mut R {
      &mut self.lock_guard
    }
  }
}

#[cfg(all(feature = "tokio", test))]
mod tests {
  use crate::pool::{fixed_pool::FixedPoolTokio, Pool, SimpleRM};

  #[tokio::test]
  async fn held_lock_is_not_modified() {
    let pool = pool();
    let lhs_lock = pool.get(&(), &()).await.unwrap();

    *pool.get(&(), &()).await.unwrap().release().await = 1;
    assert_eq!([**lhs_lock, *pool.get(&(), &()).await.unwrap().release().await], [0, 1]);

    *pool.get(&(), &()).await.unwrap().release().await = 2;
    assert_eq!([**lhs_lock, *pool.get(&(), &()).await.unwrap().release().await], [0, 2]);

    drop(lhs_lock.release().await);

    *pool.get(&(), &()).await.unwrap().release().await = 1;
    assert_eq!(
      [
        *pool.get(&(), &()).await.unwrap().release().await,
        *pool.get(&(), &()).await.unwrap().release().await
      ],
      [0, 1]
    );
  }

  fn pool() -> FixedPoolTokio<i32, SimpleRM<crate::Error, (), i32>> {
    fn cb(_: &()) -> crate::Result<i32> {
      Ok(0)
    }
    FixedPoolTokio::new(2, SimpleRM::new(cb, ()))
  }
}
