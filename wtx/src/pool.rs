//! Pool Manager

mod resource_manager;
#[cfg(feature = "std")]
mod simple_pool;

use core::future::Future;
#[cfg(feature = "postgres")]
pub use resource_manager::database::PostgresRM;
pub use resource_manager::{ResourceManager, SimpleRM};
#[cfg(feature = "std")]
pub use simple_pool::*;

/// Manages HTTP/2 resources for clients and servers.
#[cfg(feature = "http2")]
pub type Http2BufferRM<RRB> =
  SimpleRM<fn() -> Result<crate::http2::Http2Buffer<RRB>, crate::Error>>;
/// Manages resources for HTTP2 requests and responses.
#[cfg(feature = "http2")]
pub type StreamBufferRM = SimpleRM<fn() -> Result<crate::http::ReqResBuffer, crate::Error>>;
/// Manages WebSocket resources.
#[cfg(feature = "web-socket")]
pub type WebSocketRM = SimpleRM<
  fn() -> Result<
    (crate::web_socket::FrameBufferVec, crate::web_socket::WebSocketBuffer),
    crate::Error,
  >,
>;

/// A pool contains a set of resources that are behind some synchronism mechanism.
pub trait Pool {
  /// Element returned by [`Pool::get`].
  type GetElem<'this>
  where
    Self: 'this;
  /// See [`ResourceManager`].
  type ResourceManager: ResourceManager;

  /// Tries to retrieve a free resource.
  ///
  /// If the resource does not exist, a new one is created and if the pool is full, this method will
  /// await until a free resource is available.
  fn get<'this>(
    &'this self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> impl Future<
    Output = Result<Self::GetElem<'this>, <Self::ResourceManager as ResourceManager>::Error>,
  >;
}

impl<T> Pool for &T
where
  T: Pool,
{
  type GetElem<'this> = T::GetElem<'this>
  where
    Self: 'this;
  type ResourceManager = T::ResourceManager;

  #[inline]
  async fn get<'this>(
    &'this self,
    ca: &<Self::ResourceManager as ResourceManager>::CreateAux,
    ra: &<Self::ResourceManager as ResourceManager>::RecycleAux,
  ) -> Result<Self::GetElem<'this>, <Self::ResourceManager as ResourceManager>::Error> {
    (**self).get(ca, ra).await
  }
}
