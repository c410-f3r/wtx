use crate::{
  misc::{Lock, Queue, RefCounter, SyncLock},
  pool::{Pool, ResourceManager},
};
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

/// A [SimplePool] synchronized by [tokio::sync::Mutex].
#[cfg(all(feature = "parking_lot", feature = "tokio"))]
pub type SimplePoolTokio<R, RM> = SimplePool<
  alloc::sync::Arc<parking_lot::Mutex<Queue<usize>>>,
  tokio::sync::Mutex<SimplePoolResource<R>>,
  RM,
>;
/// A [SimplePoolGetElem] synchronized by [tokio::sync::MutexGuard].
#[cfg(all(feature = "parking_lot", feature = "tokio"))]
pub type SimplePoolGetElemTokio<'guard, R> = SimplePoolGetElem<
  alloc::sync::Arc<parking_lot::Mutex<Queue<usize>>>,
  tokio::sync::MutexGuard<'guard, SimplePoolResource<R>>,
>;

/// Simple pool
#[derive(Debug)]
pub struct SimplePool<IC, RL, RM> {
  indcs: IC,
  locks: Vec<RL>,
  rm: RM,
}

impl<IC, R, RL, RM> SimplePool<IC, RL, RM>
where
  IC: RefCounter + SyncLock<Resource = Queue<usize>>,
  IC::Item: SyncLock<Resource = Queue<usize>>,
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> IC: 'any,
  for<'any> R: 'any,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  /// Creates a new instance with a maximum amount of elements delimited by `len`.
  ///
  /// If `0`, then `len` will be stored as `1`.
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
        rslt.extend((0..len).map(|_| RL::new(SimplePoolResource(None))));
        rslt
      },
      rm,
    }
  }

  /// Sometimes it is desirable to eagerly initialize all instances.
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

impl<IC, R, RL, RM> Pool for SimplePool<IC, RL, RM>
where
  IC: RefCounter + SyncLock<Resource = Queue<usize>>,
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> IC: 'any,
  for<'any> R: 'any,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  type GetElem<'this> = SimplePoolGetElem<IC, RL::Guard<'this>>;
  type ResourceManager = RM;

  #[inline]
  async fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> Result<Self::GetElem<'_>, RM::Error> {
    loop {
      let mut indcs_lock = self.indcs.lock();
      let Some(idx) = indcs_lock.pop_back() else {
        drop(indcs_lock);
        continue;
      };
      drop(indcs_lock);
      let lock = self.locks.get(idx).unwrap();
      let mut resource = lock.lock().await;
      match &mut resource.0 {
        None => {
          resource.0 = Some(self.rm.create(ca).await?);
        }
        Some(resource) if self.rm.is_invalid(resource) => {
          self.rm.recycle(ra, resource).await?;
        }
        _ => {}
      }
      return Ok(SimplePoolGetElem { idx, indcs: self.indcs.clone(), resource });
    }
  }
}

/// Controls the guard locks related to [SimplePool].
#[derive(Debug, PartialEq)]
pub struct SimplePoolGetElem<I, R>
where
  I: SyncLock<Resource = Queue<usize>>,
{
  idx: usize,
  indcs: I,
  resource: R,
}

impl<I, R> Deref for SimplePoolGetElem<I, R>
where
  I: SyncLock<Resource = Queue<usize>>,
{
  type Target = R;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.resource
  }
}

impl<I, R> DerefMut for SimplePoolGetElem<I, R>
where
  I: SyncLock<Resource = Queue<usize>>,
{
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.resource
  }
}

impl<I, R> Drop for SimplePoolGetElem<I, R>
where
  I: SyncLock<Resource = Queue<usize>>,
{
  #[inline]
  fn drop(&mut self) {
    let _rslt = self.indcs.lock().push_front(self.idx);
  }
}

/// Resource related to [SimplePool].
#[derive(Debug, PartialEq)]
pub struct SimplePoolResource<T>(Option<T>);

impl<R> Deref for SimplePoolResource<R> {
  type Target = R;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_ref().unwrap()
  }
}

impl<R> DerefMut for SimplePoolResource<R> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_mut().unwrap()
  }
}

#[cfg(feature = "tokio")]
mod _tokio {
  use crate::{
    misc::{Lease, LeaseMut, Queue, SyncLock},
    pool::{SimplePoolGetElem, SimplePoolResource},
  };
  use tokio::sync::MutexGuard;

  impl<I, R> Lease<R> for SimplePoolGetElem<I, MutexGuard<'_, SimplePoolResource<R>>>
  where
    I: SyncLock<Resource = Queue<usize>>,
  {
    #[inline]
    fn lease(&self) -> &R {
      &self.resource
    }
  }

  impl<I, R> LeaseMut<R> for SimplePoolGetElem<I, MutexGuard<'_, SimplePoolResource<R>>>
  where
    I: SyncLock<Resource = Queue<usize>>,
  {
    #[inline]
    fn lease_mut(&mut self) -> &mut R {
      &mut self.resource
    }
  }
}

#[cfg(all(feature = "tokio", feature = "parking_lot", test))]
mod tests {
  use crate::pool::{simple_pool::SimplePoolTokio, Pool, SimpleRM};

  #[tokio::test]
  async fn held_lock_is_not_modified() {
    let pool = pool();
    let lhs_lock = pool.get(&(), &()).await.unwrap();

    ***pool.get(&(), &()).await.unwrap() = 1;
    assert_eq!([***lhs_lock, ***pool.get(&(), &()).await.unwrap()], [0, 1]);

    ***pool.get(&(), &()).await.unwrap() = 2;
    assert_eq!([***lhs_lock, ***pool.get(&(), &()).await.unwrap()], [0, 2]);

    drop(lhs_lock);

    ***pool.get(&(), &()).await.unwrap() = 1;
    assert_eq!(
      [***pool.get(&(), &()).await.unwrap(), ***pool.get(&(), &()).await.unwrap()],
      [0, 1]
    );
  }

  fn pool() -> SimplePoolTokio<i32, SimpleRM<crate::Error, (), i32>> {
    fn cb(_: &()) -> crate::Result<i32> {
      Ok(0)
    }
    SimplePoolTokio::new(2, SimpleRM::new(cb, ()))
  }
}
