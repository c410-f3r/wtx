use crate::{
  pool::{Pool, ResourceManager},
  sync::{Arc, Lock},
};
use alloc::vec::Vec;
use core::{
  cell::RefCell,
  future::poll_fn,
  ops::{Deref, DerefMut},
  task::{Poll, Waker},
};
use std::sync::Mutex;

/// A [`SimplePool`] synchronized by [`RefCell`].
pub type SimplePoolRefCell<RM> =
  SimplePool<RefCell<SimplePoolResource<<RM as ResourceManager>::Resource>>, RM>;
/// A [`SimplePool`] synchronized by [`tokio::sync::Mutex`].
#[cfg(feature = "tokio")]
pub type SimplePoolTokio<RM> =
  SimplePool<tokio::sync::Mutex<SimplePoolResource<<RM as ResourceManager>::Resource>>, RM>;

/// Pool with a fixed number of elements.
#[derive(Debug)]
pub struct SimplePool<RL, RM> {
  resources: Arc<PoolResources<RL, RM>>,
  state: Arc<Mutex<PoolState>>,
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
    let mut locks = Vec::with_capacity(len);
    locks.extend((0..len).map(|_| RL::new(SimplePoolResource(None))));
    Self {
      resources: Arc::new(PoolResources { locks, rm }),
      state: Arc::new(Mutex::new(PoolState {
        available: (0..len).collect(),
        wakers: Vec::with_capacity(len),
      })),
    }
  }

  /// Sometimes it is desirable to eagerly initialize all instances.
  #[inline]
  pub async fn init_all(&self, ca: &RM::CreateAux, ra: &RM::RecycleAux) -> Result<(), RM::Error> {
    for _ in 0..self.resources.locks.len() {
      let _guard = self.get(ca, ra).await?;
    }
    Ok(())
  }

  #[cfg(feature = "http-client-pool")]
  pub(crate) async fn into_for_each<FUN>(&self, mut cb: impl FnMut(R) -> FUN)
  where
    FUN: Future<Output = ()>,
  {
    for idx in 0..self.resources.locks.len() {
      if let Some(lock) = self.resources.locks.get(idx) {
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
  fn stream_aux(init: Self::Init) -> crate::Result<Self> {
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
    let idx = poll_fn(|cx| match self.state.try_lock() {
      Ok(mut elem) => {
        if let Some(idx) = elem.available.pop() {
          Poll::Ready(idx)
        } else {
          elem.wakers.push(cx.waker().clone());
          Poll::Pending
        }
      }
      Err(_) => {
        cx.waker().wake_by_ref();
        Poll::Pending
      }
    })
    .await;
    let mut drop_guard = SimplePoolGetDropGuard { state: &self.state, idx: Some(idx) };
    // SAFETY: `idx` is guaranteed to be within bounds as defined in the constructor
    let lock = unsafe { self.resources.locks.get(idx).unwrap_unchecked() };
    let mut lock_guard = lock.lock().await;
    match lock_guard.0.as_mut() {
      None => {
        lock_guard.0 = Some(self.resources.rm.create(ca).await?);
      }
      Some(elem) => {
        if self.resources.rm.is_invalid(elem) {
          self.resources.rm.recycle(ra, elem).await?;
        }
      }
    }
    let _ = drop_guard.idx.take();
    Ok(SimplePoolGetElem { state: Arc::clone(&self.state), idx, resource: lock_guard })
  }
}

impl<RL, RM> Clone for SimplePool<RL, RM> {
  #[inline]
  fn clone(&self) -> Self {
    Self { resources: Arc::clone(&self.resources), state: Arc::clone(&self.state) }
  }
}

/// Controls the guard locks related to [`SimplePool`].
#[derive(Debug)]
pub struct SimplePoolGetElem<R> {
  idx: usize,
  resource: R,
  state: Arc<Mutex<PoolState>>,
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
    push_available(self.idx, &self.state);
  }
}

/// Resource related to [`SimplePool`].
#[derive(Debug, PartialEq)]
pub struct SimplePoolResource<T>(Option<T>);

impl<T> SimplePoolResource<T> {
  /// Returns the inner element.
  #[expect(clippy::unwrap_used, reason = "public instances always have valid contents")]
  #[inline]
  pub fn into_inner(self) -> T {
    self.0.unwrap()
  }
}

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

#[derive(Debug)]
struct PoolState {
  available: Vec<usize>,
  wakers: Vec<Waker>,
}

#[derive(Debug)]
struct PoolResources<RL, RM> {
  locks: Vec<RL>,
  rm: RM,
}

#[derive(Debug)]
struct SimplePoolGetDropGuard<'any> {
  state: &'any Mutex<PoolState>,
  idx: Option<usize>,
}

impl<'a> Drop for SimplePoolGetDropGuard<'a> {
  fn drop(&mut self) {
    if let Some(idx) = self.idx {
      let mut state = self.state.lock().unwrap();
      state.available.push(idx);
      if let Some(waker) = state.wakers.pop() {
        waker.wake();
      }
    }
  }
}

fn push_available(idx: usize, state: &Mutex<PoolState>) {
  let mut state_guard = state.lock().unwrap();
  state_guard.available.push(idx);
  if let Some(waker) = state_guard.wakers.pop() {
    waker.wake();
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

#[cfg(test)]
mod tests {
  use crate::{
    executor::Runtime,
    pool::{SimpleRM, simple_pool::SimplePoolRefCell},
  };

  #[test]
  fn held_lock_is_not_modified() {
    Runtime::new()
      .block_on(async {
        let pool = pool();
        let lhs_lock = pool.get().await.unwrap();

        ***pool.get().await.unwrap() = 1;
        assert_eq!([***lhs_lock, ***pool.get().await.unwrap()], [0, 1]);

        ***pool.get().await.unwrap() = 2;
        assert_eq!([***lhs_lock, ***pool.get().await.unwrap()], [0, 2]);

        drop(lhs_lock);

        ***pool.get().await.unwrap() = 1;
        assert_eq!([***pool.get().await.unwrap(), ***pool.get().await.unwrap()], [1, 2]);
      })
      .unwrap();
  }

  fn pool() -> SimplePoolRefCell<SimpleRM<fn() -> crate::Result<i32>>> {
    SimplePoolRefCell::new(2, SimpleRM::new(|| Ok(0)))
  }
}
