//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#[macro_use]
mod macros;

mod conn_aux;
mod cors_middleware;
mod endpoint;
mod middleware;
mod param_wrappers;
mod path_management;
mod path_params;
mod redirect;
mod req_aux;
mod res_finalizer;
mod route_wrappers;
mod router;
mod server_framework_builder;
mod state;

use crate::{
  http::{ConnParams, LowLevelServer, ReqResBuffer, Request, Response},
  http2::Http2Buffer,
  misc::Rng,
};
use alloc::sync::Arc;
pub use conn_aux::ConnAux;
pub use cors_middleware::CorsMiddleware;
pub use endpoint::Endpoint;
pub use middleware::{ReqMiddleware, ResMiddleware};
pub use param_wrappers::*;
pub use path_management::PathManagement;
pub use path_params::PathParams;
pub use redirect::Redirect;
pub use req_aux::ReqAux;
pub use res_finalizer::ResFinalizer;
pub use route_wrappers::{get, json, post, Get, Json, Post};
pub use router::Router;
pub use server_framework_builder::ServerFrameworkBuilder;
pub use state::{State, StateClean};

/// Server
#[derive(Debug)]
pub struct ServerFramework<CA, CAC, E, P, RA, RAC, REQM, RESM> {
  ca_cb: CAC,
  cp: ConnParams,
  ra_cb: RAC,
  router: Arc<Router<CA, E, P, RA, REQM, RESM, ReqResBuffer>>,
}

impl<CA, CAC, E, P, RA, RAC, REQM, RESM> ServerFramework<CA, CAC, E, P, RA, RAC, REQM, RESM>
where
  CA: Clone + ConnAux + Send + 'static,
  CAC: Clone + Fn() -> CA::Init + Send + 'static,
  E: From<crate::Error> + Send + 'static,
  P: Send + 'static,
  RA: ReqAux + Send + 'static,
  RAC: Clone + Fn() -> RA::Init + Send + 'static,
  REQM: ReqMiddleware<CA, E, RA, ReqResBuffer, apply_req_middleware(..): Send> + Send + 'static,
  RESM: ResMiddleware<CA, E, RA, ReqResBuffer, apply_res_middleware(..): Send> + Send + 'static,
  Arc<Router<CA, E, P, RA, REQM, RESM, ReqResBuffer>>: Send,
  Router<CA, E, P, RA, REQM, RESM, ReqResBuffer>:
    PathManagement<CA, E, RA, ReqResBuffer, manage_path(..): Send>,
{
  /// Starts listening to incoming requests based on the given `host`.
  #[inline]
  pub async fn listen<RNG>(
    self,
    host: &str,
    rng: RNG,
    err_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()>
  where
    RNG: Clone + Rng + Send + 'static,
  {
    let Self { ca_cb, cp, ra_cb, router } = self;
    LowLevelServer::tokio_http2(
      host,
      move || Ok((CA::conn_aux(ca_cb())?, Http2Buffer::new(rng.clone()), cp.to_hp())),
      err_cb,
      Self::handle,
      move || Ok(((ra_cb.clone(), Arc::clone(&router)), ReqResBuffer::empty())),
      (|| Ok(()), |_| {}, |_, stream| async move { Ok(stream.into_split()) }),
    )
    .await
  }

  /// Starts listening to incoming encrypted requests based on the given `host`.
  #[cfg(feature = "tokio-rustls")]
  #[inline]
  pub async fn listen_tls<RNG>(
    self,
    (cert_chain, priv_key): (&'static [u8], &'static [u8]),
    host: &str,
    rng: RNG,
    err_cb: impl Clone + Fn(E) + Send + 'static,
  ) -> crate::Result<()>
  where
    RNG: Clone + Rng + Send + 'static,
  {
    let Self { ca_cb, cp, ra_cb, router } = self;
    LowLevelServer::tokio_http2(
      host,
      move || Ok((CA::conn_aux(ca_cb())?, Http2Buffer::new(rng.clone()), cp.to_hp())),
      err_cb,
      Self::handle,
      move || Ok(((ra_cb.clone(), Arc::clone(&router)), ReqResBuffer::empty())),
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

  async fn handle(
    mut ca: CA,
    (ra_cb, router): (impl Fn() -> RA::Init, Arc<Router<CA, E, P, RA, REQM, RESM, ReqResBuffer>>),
    mut req: Request<ReqResBuffer>,
  ) -> Result<Response<ReqResBuffer>, E> {
    let mut ra = RA::req_aux(ra_cb(), &mut req)?;
    let matched = router.router.at(req.rrd.uri.path()).map_err(From::from)?;
    let status_code = router.manage_path(&mut ca, (0, matched.value), &mut ra, &mut req).await?;
    Ok(Response { rrd: req.rrd, status_code, version: req.version })
  }
}

#[cfg(test)]
mod tests {
  use crate::http::{
    server_framework::{get, Router, ServerFrameworkBuilder, StateClean},
    ReqResBuffer, StatusCode,
  };

  #[tokio::test]
  async fn compiles() {
    async fn one(_: StateClean<'_, (), (), ReqResBuffer>) -> crate::Result<StatusCode> {
      Ok(StatusCode::Ok)
    }

    async fn two(_: StateClean<'_, (), (), ReqResBuffer>) -> crate::Result<StatusCode> {
      Ok(StatusCode::Ok)
    }

    let router = Router::paths(paths!(
      ("/aaa", Router::paths(paths!(("/bbb", get(one)), ("/ccc", get(two)))).unwrap()),
      ("/ddd", get(one)),
      ("/eee", get(two)),
      ("/fff", Router::paths(paths!(("/ggg", get(one)))).unwrap()),
    ))
    .unwrap();

    let _sf = ServerFrameworkBuilder::new(router).without_aux();
  }
}
