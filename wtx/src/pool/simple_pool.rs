use crate::{
  misc::{Lease, LeaseMut},
  pool::ResourceManager,
  sync::{Arc, AsyncMutex, AsyncMutexGuard, SyncMutex},
};
use alloc::vec::Vec;
use core::{
  fmt::{Debug, Formatter},
  future::poll_fn,
  iter,
  ops::{Deref, DerefMut},
  task::{Poll, Waker},
};

/// Pool with a fixed number of elements.
pub struct SimplePool<RM>
where
  RM: ResourceManager,
{
  resources: Arc<PoolResources<RM>>,
  state: Arc<SyncMutex<PoolState>>,
}

impl<RM> SimplePool<RM>
where
  RM: ResourceManager,
{
  /// Creates a new instance with a maximum amount of elements delimited by `len`.
  ///
  /// If `0`, then `len` will be stored as `1`.
  #[inline]
  pub fn new(mut len: usize, rm: RM) -> Self {
    len = len.max(1);
    let mut locks = Vec::with_capacity(len);
    locks.extend(iter::repeat_with(|| AsyncMutex::new(SimplePoolResource(None))).take(len));
    Self {
      resources: Arc::new(PoolResources { locks, rm }),
      state: Arc::new(SyncMutex::new(PoolState {
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

  /// Tries to retrieve a free resource.
  #[inline]
  pub async fn get<'guard, 'this>(
    &'this self,
    ca: &RM::CreateAux,
    ra: &RM::RecycleAux,
  ) -> Result<SimplePoolGetElem<AsyncMutexGuard<'guard, SimplePoolResource<RM::Resource>>>, RM::Error>
  where
    'this: 'guard,
  {
    let idx = poll_fn(|cx| {
      if let Some(mut elem) = self.state.try_lock() {
        if let Some(idx) = elem.available.pop() {
          Poll::Ready(idx)
        } else {
          elem.wakers.push(cx.waker().clone());
          Poll::Pending
        }
      } else {
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

  #[cfg(feature = "http-client-pool")]
  pub(crate) async fn into_for_each<FUN>(&self, mut cb: impl FnMut(RM::Resource) -> FUN)
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

impl<RM> SimplePool<RM>
where
  RM: ResourceManager<CreateAux = (), RecycleAux = ()>,
{
  /// Shortcut for implementations that don't require inputs.
  #[inline]
  pub async fn get_with_unit(
    &self,
  ) -> Result<SimplePoolGetElem<AsyncMutexGuard<'_, SimplePoolResource<RM::Resource>>>, RM::Error>
  {
    self.get(&(), &()).await
  }
}

#[cfg(feature = "http-server-framework")]
impl<RM> crate::http::server_framework::ConnAux for SimplePool<RM>
where
  RM: ResourceManager,
{
  type Init = Self;

  #[inline]
  fn conn_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}

#[cfg(feature = "http-server-framework")]
impl<RM> crate::http::server_framework::StreamAux for SimplePool<RM>
where
  RM: ResourceManager,
{
  type Init = Self;

  #[inline]
  fn stream_aux(init: Self::Init) -> crate::Result<Self> {
    Ok(init)
  }
}

impl<RM> Clone for SimplePool<RM>
where
  RM: ResourceManager,
{
  #[inline]
  fn clone(&self) -> Self {
    Self { resources: Arc::clone(&self.resources), state: Arc::clone(&self.state) }
  }
}

impl<RM> Debug for SimplePool<RM>
where
  RM: ResourceManager,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Pool").finish()
  }
}

/// Controls the guard locks related to [`SimplePool`].
#[derive(Debug)]
pub struct SimplePoolGetElem<R> {
  idx: usize,
  resource: R,
  state: Arc<SyncMutex<PoolState>>,
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
    push_available(self.idx, &self.state);
  }
}

impl<R> Lease<R> for SimplePoolGetElem<AsyncMutexGuard<'_, SimplePoolResource<R>>> {
  #[inline]
  fn lease(&self) -> &R {
    &self.resource
  }
}

impl<R> LeaseMut<R> for SimplePoolGetElem<AsyncMutexGuard<'_, SimplePoolResource<R>>> {
  #[inline]
  fn lease_mut(&mut self) -> &mut R {
    &mut self.resource
  }
}

/// Resource related to [`SimplePool`].
#[derive(Debug, PartialEq)]
pub struct SimplePoolResource<T>(Option<T>);

impl<T> SimplePoolResource<T> {
  /// Returns the inner element.
  #[expect(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "public instances always have valid contents"
  )]
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
struct PoolResources<RM>
where
  RM: ResourceManager,
{
  locks: Vec<AsyncMutex<SimplePoolResource<RM::Resource>>>,
  rm: RM,
}

struct SimplePoolGetDropGuard<'any> {
  state: &'any SyncMutex<PoolState>,
  idx: Option<usize>,
}

impl Drop for SimplePoolGetDropGuard<'_> {
  fn drop(&mut self) {
    if let Some(idx) = self.idx {
      push_available(idx, self.state);
    }
  }
}

fn push_available(idx: usize, state: &SyncMutex<PoolState>) {
  let mut state_guard = state.lock();
  state_guard.available.push(idx);
  if let Some(waker) = state_guard.wakers.pop() {
    waker.wake();
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    executor::Runtime,
    pool::{SimpleRM, simple_pool::SimplePool},
  };

  #[test]
  fn held_lock_is_not_modified() {
    Runtime::new().block_on(async {
      let pool = pool();
      let lhs_lock = pool.get_with_unit().await.unwrap();

      ***pool.get_with_unit().await.unwrap() = 1;
      assert_eq!([***lhs_lock, ***pool.get_with_unit().await.unwrap()], [0, 1]);

      ***pool.get_with_unit().await.unwrap() = 2;
      assert_eq!([***lhs_lock, ***pool.get_with_unit().await.unwrap()], [0, 2]);

      drop(lhs_lock);

      ***pool.get_with_unit().await.unwrap() = 1;
      assert_eq!(
        [***pool.get_with_unit().await.unwrap(), ***pool.get_with_unit().await.unwrap()],
        [1, 2]
      );
    });
  }

  fn pool() -> SimplePool<SimpleRM<fn() -> crate::Result<i32>>> {
    SimplePool::new(2, SimpleRM::new(|| Ok(0)))
  }
}
