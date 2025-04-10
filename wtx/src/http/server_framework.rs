//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#[macro_use]
mod macros;

mod arguments;
mod conn_aux;
mod cors_middleware;
mod endpoint;
pub(crate) mod endpoint_node;
mod methods;
mod middleware;
mod path_params;
mod redirect;
mod res_finalizer;
mod route_match;
mod router;
mod server_framework_builder;
mod server_framework_error;
mod state;
mod stream_aux;
#[cfg(all(feature = "_async-tests", feature = "matchit", test))]
mod tests;
#[cfg(all(feature = "nightly", feature = "tokio"))]
mod tokio;

use crate::{
  http::{AutoStream, OperationMode, ReqResBuffer, Response, conn_params::ConnParams},
  misc::ArrayVector,
  sync::Arc,
};
pub use arguments::*;
pub use conn_aux::ConnAux;
pub use cors_middleware::{CorsMiddleware, OriginResponse};
pub use endpoint::Endpoint;
pub use endpoint_node::EndpointNode;
pub use methods::{
  get::{Get, get},
  json::{Json, json},
  post::{Post, post},
  web_socket::{WebSocket, web_socket},
};
pub use middleware::Middleware;
pub use path_params::PathParams;
pub use redirect::Redirect;
pub use res_finalizer::ResFinalizer;
pub use route_match::RouteMatch;
pub use router::Router;
pub use server_framework_builder::ServerFrameworkBuilder;
pub use server_framework_error::ServerFrameworkError;
pub use state::{State, StateClean, StateGeneric};
pub use stream_aux::StreamAux;

/// Server
#[derive(Debug)]
pub struct ServerFramework<CA, CACB, CBP, E, EN, M, S, SA, SACB> {
  _ca_cb: CACB,
  _cbp: CBP,
  _cp: ConnParams,
  _sa_cb: SACB,
  _router: Arc<Router<CA, E, EN, M, S, SA>>,
}

impl<CA, CACB, CBP, E, EN, M, S, SA, SACB> ServerFramework<CA, CACB, CBP, E, EN, M, S, SA, SACB>
where
  E: From<crate::Error>,
  EN: EndpointNode<CA, E, S, SA>,
  M: Middleware<CA, E, SA>,
  SA: StreamAux,
{
  #[inline]
  async fn _auto(
    headers_aux: (ArrayVector<RouteMatch, 4>, Arc<Router<CA, E, EN, M, S, SA>>),
    mut auto_stream: AutoStream<CA, SA>,
  ) -> Result<Response<ReqResBuffer>, E> {
    let status_code = headers_aux.1.auto(&mut auto_stream, (0, &headers_aux.0)).await?;
    Ok(Response { rrd: auto_stream.req.rrd, status_code, version: auto_stream.req.version })
  }

  #[inline]
  fn _route_params(
    path: &str,
    router: &Arc<Router<CA, E, EN, M, S, SA>>,
  ) -> Result<(ArrayVector<RouteMatch, 4>, OperationMode), E> {
    #[cfg(feature = "matchit")]
    return Ok(router._matcher.at(path).map_err(From::from)?.value.clone());
    #[cfg(not(feature = "matchit"))]
    return Ok((
      ArrayVector::new(),
      *router._matcher.get(path).ok_or_else(|| ServerFrameworkError::UnknownPath.into())?,
    ));
  }
}
