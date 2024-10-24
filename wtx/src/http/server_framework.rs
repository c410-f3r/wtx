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
#[cfg(feature = "nightly")]
mod tokio;

use crate::http::{conn_params::ConnParams, ReqResBuffer, Request, Response};
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
pub use state::{State, StateClean, StateGeneric};

/// Server
#[derive(Debug)]
pub struct ServerFramework<CA, CAC, E, P, RA, RAC, REQM, RESM> {
  _ca_cb: CAC,
  _cp: ConnParams,
  _ra_cb: RAC,
  _router: Arc<Router<CA, E, P, RA, REQM, RESM>>,
}

impl<CA, CAC, E, P, RA, RAC, REQM, RESM> ServerFramework<CA, CAC, E, P, RA, RAC, REQM, RESM>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, RA>,
  RA: ReqAux,
  REQM: ReqMiddleware<CA, E, RA>,
  RESM: ResMiddleware<CA, E, RA>,
{
  async fn _auto(
    mut ca: CA,
    (ra_cb, router): (impl Fn() -> RA::Init, Arc<Router<CA, E, P, RA, REQM, RESM>>),
    mut req: Request<ReqResBuffer>,
  ) -> Result<Response<ReqResBuffer>, E> {
    let mut ra = RA::req_aux(ra_cb(), &mut req)?;
    #[cfg(feature = "matchit")]
    let num = router.router.at(req.rrd.uri.path()).map_err(From::from)?.value;
    #[cfg(not(feature = "matchit"))]
    let num = &[];
    let status_code = router.manage_path(&mut ca, (0, num), &mut ra, &mut req).await?;
    Ok(Response { rrd: req.rrd, status_code, version: req.version })
  }
}

#[cfg(all(feature = "_async-tests", test))]
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
