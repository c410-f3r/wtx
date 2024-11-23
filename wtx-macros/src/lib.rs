//! WTX - Macros

#![expect(clippy::too_many_lines, reason = "Unimportant")]

mod client_api_framework;
mod error;
mod http;
mod misc;

use error::Error;

type Result<T> = core::result::Result<T, Error>;

/// API Parameters
///
/// Creates types referring an API and its possible de-serializers/serializers or transport
/// variants.
#[proc_macro_attribute]
pub fn api_params(
  attrs: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  match client_api_framework::api_params::api_params(attrs, item) {
    Err(err) => syn::Error::from(err).to_compile_error().into(),
    Ok(elem) => elem,
  }
}

/// Connection Auxiliary
#[proc_macro_derive(ConnAux)]
pub fn conn_aux(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  match http::conn_aux(item) {
    Err(err) => syn::Error::from(err).to_compile_error().into(),
    Ok(elem) => elem,
  }
}

/// Package
///
/// A framework-like attribute placed in inline modules that creates all the mandatory elements
/// and optional elements related to `wtx::pkg::Package`.
///
/// ```rust
/// struct SomeApi;
///
/// #[wtx_macros::pkg(api(SomeApi), data_format(json_rpc("SomeEndpoint")))]
/// mod pkg {
///   #[pkg::req_data]
///   pub struct SomeEndpointReq<'string> {
///     ping: &'string str,
///   }
///
///   #[pkg::res_data]
///   pub struct SomeEndpointRes {
///     pong: String,
///   }
/// }
/// ```
#[proc_macro_attribute]
pub fn pkg(
  attr: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  match client_api_framework::pkg::pkg(attr, item) {
    Err(err) => syn::Error::from(err).to_compile_error().into(),
    Ok(elem) => elem,
  }
}
