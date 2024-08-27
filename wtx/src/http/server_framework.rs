//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#[macro_use]
mod macros;

mod endpoint;
mod middlewares;
mod param_wrappers;
mod path_management;
mod path_params;
mod response_finalizer;
mod route_wrappers;
mod router;

use crate::{
  http::{ConnParams, LowLevelServer, ReqResBuffer, Request, Response},
  http2::Http2Buffer,
  misc::StdRng,
};
use alloc::sync::Arc;
use core::{fmt::Debug, marker::PhantomData};
pub use endpoint::Endpoint;
pub use middlewares::{ReqMiddlewares, ResMiddlewares};
pub use param_wrappers::{PathOwned, PathStr, SerdeJson};
pub use path_management::PathManagement;
pub use path_params::PathParams;
pub use response_finalizer::ResponseFinalizer;
pub use route_wrappers::{get, json, post, Get, Json, Post};
pub use router::Router;

/// Server
#[derive(Debug)]
pub struct ServerFramework<E, P, REQM, RESM> {
  pub(crate) cp: ConnParams,
  phantom: PhantomData<fn() -> E>,
  router: Arc<Router<P, REQM, RESM>>,
}

impl<E, P, REQM, RESM> ServerFramework<E, P, REQM, RESM>
where
  E: From<crate::Error> + Send + 'static,
  P: Send + 'static,
  REQM: ReqMiddlewares<E, ReqResBuffer, apply_req_middlewares(..): Send> + Send + 'static,
  RESM: ResMiddlewares<E, ReqResBuffer, apply_res_middlewares(..): Send> + Send + 'static,
  Arc<Router<P, REQM, RESM>>: Send,
  Router<P, REQM, RESM>: PathManagement<E, ReqResBuffer, manage_path(..): Send>,
{
  /// Creates a new instance with default parameters.
  #[inline]
  pub fn new(router: Router<P, REQM, RESM>) -> Self {
    Self { cp: ConnParams::default(), phantom: PhantomData, router: Arc::new(router) }
  }

  /// Starts listening to incoming requests based on the given `host`.
  #[inline]
  pub async fn listen(
    self,
    host: &str,
    err_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()> {
    let local_cp = self.cp;
    LowLevelServer::tokio_http2(
      Arc::clone(&self.router),
      host,
      err_cb,
      Self::handle,
      || Ok(Http2Buffer::new(StdRng::default())),
      move || local_cp.to_hp(),
      || Ok(ReqResBuffer::default()),
      (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
    )
    .await
  }

  /// Starts listening to incoming encrypted requests based on the given `host`.
  #[cfg(feature = "tokio-rustls")]
  #[inline]
  pub async fn listen_tls(
    self,
    (cert_chain, priv_key): (&'static [u8], &'static [u8]),
    host: &str,
    err_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()> {
    LowLevelServer::tokio_http2(
      Arc::clone(&self.router),
      host,
      err_cb,
      Self::handle,
      || Ok(Http2Buffer::new(StdRng::default())),
      move || self.cp.to_hp(),
      || Ok(ReqResBuffer::default()),
      (
        || {
          crate::misc::TokioRustlsAcceptor::without_client_auth()
            .http2()
            .build_with_cert_chain_and_priv_key(cert_chain, priv_key)
        },
        |acceptor| acceptor.clone(),
        |acceptor, stream| async move { Ok(tokio::io::split(acceptor.accept(stream).await?)) },
      ),
    )
    .await
  }

  _conn_params_methods!(cp);

  async fn handle(
    mut req: Request<ReqResBuffer>,
    router: Arc<Router<P, REQM, RESM>>,
  ) -> Result<Response<ReqResBuffer>, E> {
    let status_code = router.manage_path(true, "", &mut req, [0, 0]).await?;
    Ok(Response { rrd: req.rrd, status_code, version: req.version })
  }
}

#[cfg(test)]
mod tests {
  use crate::http::{
    server_framework::{get, Router, ServerFramework},
    ReqResBuffer, Request, StatusCode,
  };

  #[tokio::test]
  async fn compiles() {
    async fn one(_: &mut Request<ReqResBuffer>) -> crate::Result<StatusCode> {
      Ok(StatusCode::Ok)
    }

    async fn two(_: &mut Request<ReqResBuffer>) -> crate::Result<StatusCode> {
      Ok(StatusCode::Ok)
    }

    let router = Router::paths(paths!(
      ("aaa", Router::paths(paths!(("bbb", get(one)), ("ccc", get(two))))),
      ("ddd", get(one)),
      ("eee", get(two)),
      ("fff", Router::paths(paths!(("ggg", get(one))))),
    ));

    let _sf = ServerFramework::new(router);
  }
}
