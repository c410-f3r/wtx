use crate::http::{
  server_framework::{PathManagement, Router, ServerFramework},
  ReqResBuffer,
};
use alloc::sync::Arc;

/// Listens to requests and servers gRPC responses.
#[derive(Debug)]
pub struct Server<P> {
  sf: ServerFramework<crate::Error, P, (), ()>,
}

impl<P> Server<P>
where
  P: Send + 'static,
  Arc<Router<P, (), ()>>: Send,
  Router<P, (), ()>: PathManagement<crate::Error, ReqResBuffer, manage_path(..): Send>,
{
  /// Creates a new instance with default parameters.
  #[inline]
  pub fn new(router: Router<P, (), ()>) -> Self {
    Self { sf: ServerFramework::new(router) }
  }

  /// Starts listening to incoming requests based on the given `host`.
  #[inline]
  pub async fn listen(
    self,
    host: &str,
    err_cb: impl Copy + Fn(crate::Error) + Send + 'static,
  ) -> crate::Result<()> {
    self.sf.listen(host, err_cb).await
  }

  /// Starts listening to incoming encrypted requests based on the given `host`.
  #[cfg(feature = "tokio-rustls")]
  #[inline]
  pub async fn listen_tls(
    self,
    (cert_chain, priv_key): (&'static [u8], &'static [u8]),
    host: &str,
    err_cb: impl Copy + Fn(crate::Error) + Send + 'static,
  ) -> crate::Result<()> {
    self.sf.listen_tls((cert_chain, priv_key), host, err_cb).await
  }

  _conn_params_methods!(sf.cp);
}
