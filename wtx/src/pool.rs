//! Pool Manager

mod resource_manager;
mod simple_pool;

use core::{future::Future, ops::DerefMut};
#[cfg(feature = "database")]
pub use resource_manager::database::PostgresRM;
#[cfg(feature = "http2")]
pub use resource_manager::http2::{Http2ClientBufferRM, Http2ServerBufferRM, StreamBufferRM};
#[cfg(feature = "web-socket")]
pub use resource_manager::web_socket::WebSocketRM;
pub use resource_manager::{ResourceManager, SimpleRM};
pub use simple_pool::*;

/// A pool contains a set of resources that are behind some synchronism mechanism.
pub trait Pool: Sized {
  /// Result of the [Pool:get] method.
  type GetElem<'this>: DerefMut
  where
    Self: 'this;
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
  ) -> impl Future<Output = Result<Self::GetElem<'_>, <Self::ResourceManager as ResourceManager>::Error>>;
}

impl<T> Pool for &T
where
  T: Pool,
{
  type GetElem<'guard> = T::GetElem<'guard>
  where
    Self: 'guard;
  type ResourceManager = T::ResourceManager;

  #[inline]
  fn get(
    &self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> impl Future<Output = Result<Self::GetElem<'_>, <Self::ResourceManager as ResourceManager>::Error>>
  {
    (**self).get(ca, ra)
  }
}
