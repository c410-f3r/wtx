use crate::{
  misc::{Lock, PollOnce},
  pool::{Pool, ResourceManager},
};
use alloc::{sync::Arc, vec::Vec};
use core::{
  ops::{Deref, DerefMut},
  pin::pin,
  sync::atomic::{AtomicUsize, Ordering},
};

/// A [SimplePool] synchronized by [`tokio::sync::Mutex`].
#[cfg(feature = "tokio")]
pub type SimplePoolTokio<RM> =
  SimplePool<tokio::sync::Mutex<SimplePoolResource<<RM as ResourceManager>::Resource>>, RM>;
/// A [SimplePoolGetElem] synchronized by [`tokio::sync::MutexGuard`].
#[cfg(feature = "tokio")]
pub type SimplePoolGetElemTokio<'guard, R> =
  SimplePoolGetElem<tokio::sync::MutexGuard<'guard, SimplePoolResource<R>>>;

/// Simple pool that never de-allocates its elements.
#[derive(Debug)]
pub struct SimplePool<RL, RM> {
  lowest_available_idx: Arc<AtomicUsize>,
  locks: Vec<RL>,
  rm: RM,
}

impl<R, RL, RM> SimplePool<RL, RM>
where
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<Resource = R>,
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
      lowest_available_idx: Arc::new(AtomicUsize::new(0)),
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

impl<R, RL, RM> Pool for SimplePool<RL, RM>
where
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> R: 'any,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  type GetElem<'this> = SimplePoolGetElem<RL::Guard<'this>>;
  type ResourceManager = RM;

  #[allow(
    // `locks` will never be zero
    clippy::arithmetic_side_effects,
  )]
  #[inline]
  async fn get<'this>(
    &'this self,
    ca: &RM::CreateAux,
    ra: &RM::RecycleAux,
  ) -> Result<Self::GetElem<'this>, <Self::ResourceManager as ResourceManager>::Error> {
    let mut idx = self.lowest_available_idx.load(Ordering::Relaxed);
    let mut resource = loop {
      let Some(lock) = self.locks.get(idx) else {
        continue;
      };
      let Some(resource) = PollOnce(pin!(lock.lock())).await else {
        idx = idx.wrapping_add(1) % self.locks.len();
        continue;
      };
      break resource;
    };
    match &mut resource.0 {
      None => {
        resource.0 = Some(self.rm.create(ca).await?);
      }
      Some(resource) if self.rm.is_invalid(resource) => {
        self.rm.recycle(ra, resource).await?;
      }
      _ => {}
    }
    Ok(SimplePoolGetElem {
      idx,
      lowest_available_idx: Arc::clone(&self.lowest_available_idx),
      resource,
    })
  }
}

/// Controls the guard locks related to [SimplePool].
#[derive(Debug)]
pub struct SimplePoolGetElem<R> {
  idx: usize,
  lowest_available_idx: Arc<AtomicUsize>,
  resource: R,
}

impl<R> Deref for SimplePoolGetElem<R> {
  type Target = R;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.resource
  }
}

impl<R> DerefMut for SimplePoolGetElem<R> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.resource
  }
}

impl<R> Drop for SimplePoolGetElem<R> {
  #[inline]
  fn drop(&mut self) {
    let _ = self.lowest_available_idx.fetch_min(self.idx, Ordering::Relaxed);
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
    misc::{Lease, LeaseMut},
    pool::{SimplePoolGetElem, SimplePoolResource},
  };
  use tokio::sync::MutexGuard;

  impl<R> Lease<R> for SimplePoolGetElem<MutexGuard<'_, SimplePoolResource<R>>> {
    #[inline]
    fn lease(&self) -> &R {
      &self.resource
    }
  }

  impl<R> LeaseMut<R> for SimplePoolGetElem<MutexGuard<'_, SimplePoolResource<R>>> {
    #[inline]
    fn lease_mut(&mut self) -> &mut R {
      &mut self.resource
    }
  }
}

#[cfg(all(feature = "tokio", test))]
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
      [1, 2]
    );
  }

  fn pool() -> SimplePoolTokio<SimpleRM<fn() -> crate::Result<i32>>> {
    SimplePoolTokio::new(2, SimpleRM::new(|| Ok(0)))
  }
}
