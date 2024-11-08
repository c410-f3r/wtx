use crate::{
  misc::Lock,
  pool::{Pool, ResourceManager},
};
use alloc::{sync::Arc, vec::Vec};
use core::{
  future::{poll_fn, Future},
  ops::{Deref, DerefMut},
  task::{Poll, Waker},
};
use std::sync::Mutex;

/// A [`SimplePool`] synchronized by [`tokio::sync::Mutex`].
#[cfg(feature = "tokio")]
pub type SimplePoolTokio<RM> =
  SimplePool<tokio::sync::Mutex<SimplePoolResource<<RM as ResourceManager>::Resource>>, RM>;

/// Pool with a fixed number of elements.
#[derive(Debug)]
pub struct SimplePool<RL, RM> {
  available_idxs: Arc<Mutex<Vec<usize>>>,
  #[expect(clippy::rc_buffer, reason = "false-positive")]
  locks: Arc<Vec<RL>>,
  rm: Arc<RM>,
  waker: Arc<Mutex<Vec<Waker>>>,
}

impl<R, RL, RM> SimplePool<RL, RM>
where
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<Resource = R>,
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
      available_idxs: Arc::new(Mutex::new((0..len).collect())),
      locks: {
        let mut rslt = Vec::with_capacity(len);
        rslt.extend((0..len).map(|_| RL::new(SimplePoolResource(None))));
        Arc::new(rslt)
      },
      rm: Arc::new(rm),
      waker: Arc::new(Mutex::new(Vec::new())),
    }
  }

  /// Sometimes it is desirable to eagerly initialize all instances.
  #[inline]
  pub async fn init_all(&self, ca: &RM::CreateAux, ra: &RM::RecycleAux) -> Result<(), RM::Error> {
    for _ in 0..self.locks.len() {
      let _guard = self.get(ca, ra).await?;
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn _into_for_each<FUN>(&self, mut cb: impl FnMut(R) -> FUN)
  where
    FUN: Future<Output = ()>,
  {
    for idx in 0..self.locks.len() {
      if let Some(lock) = self.locks.get(idx) {
        let mut resource = lock.lock().await;
        if let Some(elem) = resource.0.take() {
          cb(elem).await;
        }
      }
    }
  }
}

impl<R, RL, RM> SimplePool<RL, RM>
where
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<CreateAux = (), RecycleAux = (), Resource = R>,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  /// Shortcut for implementations that don't require inputs.
  #[inline]
  pub async fn get(&self) -> Result<<Self as Pool>::GetElem<'_>, RM::Error> {
    <Self as Pool>::get(self, &(), &()).await
  }
}

#[cfg(feature = "http-server-framework")]
impl<RL, RM> crate::http::server_framework::ConnAux for SimplePool<RL, RM> {
  type Init = Self;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}
#[cfg(feature = "http-server-framework")]
impl<RL, RM> crate::http::server_framework::StreamAux for SimplePool<RL, RM> {
  type Init = Self;

  #[inline]
  fn req_aux(
    init: Self::Init,
    _: &mut crate::http::Request<crate::http::ReqResBuffer>,
  ) -> crate::Result<Self> {
    Ok(init)
  }
}

impl<R, RL, RM> Pool for SimplePool<RL, RM>
where
  RL: Lock<Resource = SimplePoolResource<R>>,
  RM: ResourceManager<Resource = R>,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  type GetElem<'this> = SimplePoolGetElem<RL::Guard<'this>>;
  type ResourceManager = RM;

  #[expect(clippy::unwrap_used, reason = "poisoning is ignored")]
  #[inline]
  async fn get<'this>(
    &'this self,
    ca: &RM::CreateAux,
    ra: &RM::RecycleAux,
  ) -> Result<Self::GetElem<'this>, RM::Error> {
    let (idx, lock) = poll_fn(|ctx| {
      if let Some((idx, lock)) = self.available_idxs.lock().ok().and_then(|mut el| {
        let idx = el.pop()?;
        Some((idx, self.locks.get(idx)?))
      }) {
        Poll::Ready((idx, lock))
      } else {
        self.waker.lock().unwrap().push(ctx.waker().clone());
        Poll::Pending
      }
    })
    .await;
    let mut resource = lock.lock().await;
    match &mut resource.0 {
      None => {
        resource.0 = Some(self.rm.create(ca).await?);
      }
      Some(elem) => {
        if self.rm.is_invalid(elem).await {
          self.rm.recycle(ra, elem).await?;
        }
      }
    }
    Ok(SimplePoolGetElem {
      available_idxs: Arc::clone(&self.available_idxs),
      idx,
      resource,
      waker: Arc::clone(&self.waker),
    })
  }
}

impl<RL, RM> Clone for SimplePool<RL, RM> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      available_idxs: Arc::clone(&self.available_idxs),
      locks: Arc::clone(&self.locks),
      rm: Arc::clone(&self.rm),
      waker: Arc::clone(&self.waker),
    }
  }
}

/// Controls the guard locks related to [`SimplePool`].
#[derive(Debug)]
pub struct SimplePoolGetElem<R> {
  available_idxs: Arc<Mutex<Vec<usize>>>,
  idx: usize,
  resource: R,
  waker: Arc<Mutex<Vec<Waker>>>,
}

impl<R> SimplePoolGetElem<R> {
  #[inline]
  pub(crate) fn _idx(&self) -> usize {
    self.idx
  }
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
  #[expect(clippy::unwrap_used, reason = "poisoning is ignored")]
  #[inline]
  fn drop(&mut self) {
    self.available_idxs.lock().unwrap().push(self.idx);
    for waker in self.waker.lock().unwrap().drain(..) {
      waker.wake();
    }
  }
}

/// Resource related to [`SimplePool`].
#[derive(Debug, PartialEq)]
pub struct SimplePoolResource<T>(Option<T>);

impl<R> Deref for SimplePoolResource<R> {
  type Target = R;

  #[expect(clippy::unwrap_used, reason = "public instances always have valid contents")]
  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.as_ref().unwrap()
  }
}

impl<R> DerefMut for SimplePoolResource<R> {
  #[expect(clippy::unwrap_used, reason = "public instances always have valid contents")]
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

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::pool::{simple_pool::SimplePoolTokio, SimpleRM};

  #[tokio::test]
  async fn held_lock_is_not_modified() {
    let pool = pool();
    let lhs_lock = pool.get().await.unwrap();

    ***pool.get().await.unwrap() = 1;
    assert_eq!([***lhs_lock, ***pool.get().await.unwrap()], [0, 1]);

    ***pool.get().await.unwrap() = 2;
    assert_eq!([***lhs_lock, ***pool.get().await.unwrap()], [0, 2]);

    drop(lhs_lock);

    ***pool.get().await.unwrap() = 1;
    assert_eq!([***pool.get().await.unwrap(), ***pool.get().await.unwrap()], [1, 2]);
  }

  fn pool() -> SimplePoolTokio<SimpleRM<fn() -> crate::Result<i32>>> {
    SimplePoolTokio::new(2, SimpleRM::new(|| Ok(0)))
  }
}
