//! Tools and libraries that make it easier to write, maintain, and scale web applications.

#[macro_use]
mod macros;

mod arguments;
mod body_clean;
mod conn_aux;
mod cors_middleware;
mod endpoint;
pub(crate) mod endpoint_node;
mod headers_clean;
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
mod verbatim_params;

use crate::{http::conn_params::ConnParams, sync::Arc};
pub use arguments::*;
pub use body_clean::*;
pub use conn_aux::ConnAux;
pub use cors_middleware::{CorsMiddleware, OriginResponse};
pub use endpoint::Endpoint;
pub use endpoint_node::EndpointNode;
pub use headers_clean::*;
pub use methods::{
  delete::{Delete, delete},
  get::{Get, get},
  json::{Json, json},
  patch::{Patch, patch},
  post::{Post, post},
  put::{Put, put},
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
pub use verbatim_params::VerbatimParams;

/// Server
#[derive(Debug)]
pub struct ServerFramework<CA, CACB, CBP, E, EN, M, S, SA, SACB> {
  _ca_cb: CACB,
  _cbp: CBP,
  _cp: ConnParams,
  _sa_cb: SACB,
  _router: Arc<Router<CA, E, EN, M, S, SA>>,
}
