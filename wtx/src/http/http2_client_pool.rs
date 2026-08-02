//! Structures used to construct a pool of HTTP connections

mod http2_client_pool_builder;
mod http2_client_pool_resource;
mod http2_rm;
#[cfg(all(feature = "_integration-tests", test))]
mod integration_tests;

use crate::{
  net::UriRef,
  pool::{ResourceManager, SimplePool, SimplePoolGetElem, SimplePoolResource},
  sync::AsyncMutexGuard,
};
pub use http2_client_pool_builder::Http2ClientPoolBuilder;
pub use http2_client_pool_resource::Http2ClientPoolResource;
pub use http2_rm::Http2RM;

/// An optioned pool of different HTTP connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Debug)]
pub struct Http2ClientPool<AUX, EX, TCX>
where
  Http2RM<AUX, EX, TCX>: ResourceManager,
{
  pool: SimplePool<Http2RM<AUX, EX, TCX>>,
}

impl<AUX, EX, TCX, R> Http2ClientPool<AUX, EX, TCX>
where
  Http2RM<AUX, EX, TCX>:
    ResourceManager<CreateAux = str, Error = crate::Error, RecycleAux = str, Resource = R>,
{
  /// Returns a guard that contains the internal elements.
  #[inline]
  pub async fn lock<'this>(
    &'this self,
    uri: &UriRef<'_>,
  ) -> crate::Result<SimplePoolGetElem<AsyncMutexGuard<'this, SimplePoolResource<R>>>>
  where
    R: 'this,
  {
    self.pool.get(uri.as_str(), uri.as_str()).await
  }
}
