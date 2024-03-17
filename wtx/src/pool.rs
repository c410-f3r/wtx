//! Pool Manager

mod resource_manager;
mod static_pool;

use crate::misc::LockGuard;
use alloc::boxed::Box;
use core::future::Future;
#[cfg(feature = "database")]
pub use resource_manager::database::PostgresRM;
#[cfg(feature = "web-socket")]
pub use resource_manager::web_socket::WebSocketRM;
pub use resource_manager::{ResourceManager, SimpleRM};
pub use static_pool::*;

/// A pool contains a set of resources that are behind some synchronism mechanism.
pub trait Pool: Sized {
  /// Synchronization guard.
  type Guard<'lock>: LockGuard<'lock, <Self::ResourceManager as ResourceManager>::Resource>
  where
    Self: 'lock;
  /// See [ResourceManager].
  type ResourceManager: ResourceManager;

  /// Initializes inner elements.
  fn new(rm: Self::ResourceManager) -> Self;

  /// Tries to retrieve a free resource.
  ///
  /// If the resource does not exist, a new one is created and if the pool is full, this method will
  /// await until a free resource is available.
  fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> impl Future<Output = Result<Self::Guard<'_>, <Self::ResourceManager as ResourceManager>::Error>>;
}

impl<T> Pool for Box<T>
where
  T: Pool,
{
  type Guard<'lock> = T::Guard<'lock>
  where
    <Self::ResourceManager as ResourceManager>::Resource: 'lock,
    Self: 'lock;

  type ResourceManager = T::ResourceManager;

  #[inline]
  fn new(rm: Self::ResourceManager) -> Self {
    T::new(rm).into()
  }

  #[inline]
  fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> impl Future<Output = Result<Self::Guard<'_>, <Self::ResourceManager as ResourceManager>::Error>>
  {
    (**self).get(ca, ra)
  }
}
