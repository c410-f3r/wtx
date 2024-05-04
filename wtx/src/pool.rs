//! Pool Manager

mod fixed_pool;
mod resource_manager;

use crate::misc::LockGuard;
use core::{future::Future, ops::DerefMut};
pub use fixed_pool::*;
#[cfg(feature = "database")]
pub use resource_manager::database::PostgresRM;
#[cfg(feature = "http2")]
pub use resource_manager::http2::{Http2ClientBufferRM, Http2ServerBufferRM, ReqResBufferRM};
#[cfg(feature = "web-socket")]
pub use resource_manager::web_socket::WebSocketRM;
pub use resource_manager::{ResourceManager, SimpleRM};

/// A pool contains a set of resources that are behind some synchronism mechanism.
pub trait Pool: Sized {
  /// Result of the [Pool:get] method.
  type GetRslt<'guard>: DerefMut<Target = Self::Guard<'guard>>
  where
    Self: 'guard;
  /// Synchronization mechanism.
  type Guard<'guard>: LockGuard<'guard, Self::GuardElement>
  where
    Self: 'guard;
  /// The element guarded by the synchronization mechanism.
  type GuardElement;
  /// See [ResourceManager].
  type ResourceManager: ResourceManager;

  /// Tries to retrieve a free resource.
  ///
  /// If the resource does not exist, a new one is created and if the pool is full, this method will
  /// await until a free resource is available.
  fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> impl Future<Output = Result<Self::GetRslt<'_>, <Self::ResourceManager as ResourceManager>::Error>>;
}

impl<T> Pool for &T
where
  T: Pool,
{
  type GetRslt<'guard> = T::GetRslt<'guard>
  where
    Self: 'guard;
  type Guard<'guard> = T::Guard<'guard>
  where
    Self: 'guard;
  type GuardElement = T::GuardElement;
  type ResourceManager = T::ResourceManager;

  #[inline]
  fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> impl Future<Output = Result<Self::GetRslt<'_>, <Self::ResourceManager as ResourceManager>::Error>>
  {
    (**self).get(ca, ra)
  }
}
