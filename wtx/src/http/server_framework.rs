//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#[macro_use]
mod macros;

mod middlewares;
mod path;
mod path_fun;
mod path_management;
mod paths;
mod router;
mod wrappers;

use crate::{
  http::{ConnParams, LowLevelServer, ReqResBuffer, Request, Response},
  http2::Http2Buffer,
  misc::StdRng,
};
use alloc::sync::Arc;
use core::{fmt::Debug, marker::PhantomData};
pub use middlewares::{ReqMiddlewares, ResMiddlewares};
pub use path::Path;
pub use path_fun::PathFun;
pub use path_management::PathManagement;
pub use paths::Paths;
pub use router::Router;
pub use wrappers::{get, json, post, Get, Json, Post};

/// Server
#[derive(Debug)]
pub struct ServerFramework<E, P, REQM, RESM> {
  pub(crate) cp: ConnParams,
  phantom: PhantomData<fn() -> E>,
  router: Arc<Router<P, REQM, RESM>>,
}

impl<E, P, REQM, RESM> ServerFramework<E, P, REQM, RESM>
where
  E: Debug + From<crate::Error> + Send + 'static,
  P: Send + 'static,
  REQM: ReqMiddlewares<E, ReqResBuffer> + Send + 'static,
  RESM: ResMiddlewares<E, ReqResBuffer> + Send + 'static,
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
    err_cb: impl Copy + Fn(E) + Send + 'static,
  ) -> crate::Result<()> {
    LowLevelServer::tokio_http2(
      Arc::clone(&self.router),
      host,
      err_cb,
      Self::handle,
      || Ok(Http2Buffer::new(StdRng::default())),
      move || self.cp.to_hp(),
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
    err_cb: impl Copy + Fn(E) + Send + 'static,
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
    (router, req): (Arc<Router<P, REQM, RESM>>, Request<ReqResBuffer>),
  ) -> Result<Response<ReqResBuffer>, E> {
    router.manage_path(true, "", req, [0, 0]).await
  }
}

#[cfg(test)]
mod tests {
  use crate::http::{
    server_framework::{get, Router, ServerFramework},
    ReqResBuffer, Request, Response, StatusCode,
  };

  #[tokio::test]
  async fn compiles() {
    async fn one(req: Request<ReqResBuffer>) -> crate::Result<Response<ReqResBuffer>> {
      Ok(req.into_response(StatusCode::Ok))
    }

    async fn two(req: Request<ReqResBuffer>) -> crate::Result<Response<ReqResBuffer>> {
      Ok(req.into_response(StatusCode::Ok))
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
