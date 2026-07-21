//! Structures used to construct a pool of HTTP connections

mod http2_client_pool_builder;
mod http2_client_pool_resource;
mod http2_rm;
#[cfg(all(feature = "_integration-tests", test))]
mod integration_tests;

use crate::{
  http2::Http2,
  net::UriRef,
  pool::{ResourceManager, SimplePool, SimplePoolGetElem, SimplePoolResource},
  sync::AsyncMutexGuard,
};
pub use http2_client_pool_builder::Http2ClientPoolBuilder;
pub use http2_client_pool_resource::Http2ClientPoolResource;
pub use http2_rm::Http2RM;

pub(crate) type Http2Resource<SW, TM> = Http2ClientPoolResource<Http2<SW, TM, true>>;

/// An optioned pool of different HTTP connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Debug)]
pub struct Http2ClientPool<EX, TM>
where
  Http2RM<EX, TM>: ResourceManager,
{
  pool: SimplePool<Http2RM<EX, TM>>,
}

impl<EX, TM, R> Http2ClientPool<EX, TM>
where
  Http2RM<EX, TM>:
    ResourceManager<CreateAux = str, Error = crate::Error, RecycleAux = str, Resource = R>,
{
  /// Returns a guard that contains the internal elements.
  #[inline]
  pub(crate) async fn lock<'this>(
    &'this self,
    uri: &UriRef<'_>,
  ) -> crate::Result<SimplePoolGetElem<AsyncMutexGuard<'this, SimplePoolResource<R>>>>
  where
    R: 'this,
  {
    self.pool.get(uri.as_str(), uri.as_str()).await
  }
}
