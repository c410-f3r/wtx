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
mod res_finalizer;
mod route_wrappers;
mod router;
mod server_framework_builder;
mod state;
mod stream_aux;
#[cfg(feature = "nightly")]
mod tokio;

use crate::http::{conn_params::ConnParams, AutoStream, ReqResBuffer, Response};
use alloc::sync::Arc;
pub use conn_aux::ConnAux;
pub use cors_middleware::CorsMiddleware;
pub use endpoint::Endpoint;
pub use middleware::{ReqMiddleware, ResMiddleware};
pub use param_wrappers::*;
pub use path_management::PathManagement;
pub use path_params::PathParams;
pub use redirect::Redirect;
pub use res_finalizer::ResFinalizer;
pub use route_wrappers::{get, json, post, Get, Json, Post};
pub use router::Router;
pub use server_framework_builder::ServerFrameworkBuilder;
pub use state::{State, StateClean, StateGeneric};
pub use stream_aux::StreamAux;

/// Server
#[derive(Debug)]
pub struct ServerFramework<CA, CAC, E, P, REQM, RESM, SA, SAC> {
  _ca_cb: CAC,
  _cp: ConnParams,
  _sa_cb: SAC,
  _router: Arc<Router<CA, E, P, REQM, RESM, SA>>,
}

impl<CA, CAC, E, P, REQM, RESM, SA, SAC> ServerFramework<CA, CAC, E, P, REQM, RESM, SA, SAC>
where
  E: From<crate::Error>,
  P: PathManagement<CA, E, SA>,
  REQM: ReqMiddleware<CA, E, SA>,
  RESM: ResMiddleware<CA, E, SA>,
  SA: StreamAux,
{
  async fn _auto(
    mut _as: AutoStream<CA, (impl Fn() -> SA::Init, Arc<Router<CA, E, P, REQM, RESM, SA>>)>,
  ) -> Result<Response<ReqResBuffer>, E> {
    let mut sa = SA::req_aux(_as.sa.0(), &mut _as.req)?;
    #[cfg(feature = "matchit")]
    let num = _as.sa.1.router.at(_as.req.rrd.uri.path()).map_err(From::from)?.value;
    #[cfg(not(feature = "matchit"))]
    let num = &[];
    let status_code = _as.sa.1.manage_path(&mut _as.ca, (0, num), &mut sa, &mut _as.req).await?;
    Ok(Response { rrd: _as.req.rrd, status_code, version: _as.req.version })
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
