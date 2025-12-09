//! WTX - Macros

#![expect(clippy::too_many_lines, reason = "Unimportant")]

mod client_api_framework;
mod error;
mod executor;
mod from_records;
mod from_vars;
mod http;
mod misc;
mod table;

use error::Error;

type Result<T> = core::result::Result<T, Error>;

/// API
///
/// Creates types referring an API and its possible de-serializers/serializers or transport
/// variants.
#[proc_macro_attribute]
pub fn api(
  attrs: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  match client_api_framework::api::api(attrs, item) {
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

/// From records
#[proc_macro_derive(FromRecords, attributes(from_records))]
pub fn from_records(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  match from_records::from_records(item) {
    Err(err) => syn::Error::from(err).to_compile_error().into(),
    Ok(elem) => elem,
  }
}

/// Implements the `FromVars` trait.
#[proc_macro_derive(FromVars, attributes(from_vars))]
pub fn from_vars(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  match from_vars::from_vars(item) {
    Err(err) => syn::Error::from(err).to_compile_error().into(),
    Ok(elem) => elem,
  }
}

/// Allows the execution of asynchronous programs using the runtime provided by `WTX`.
#[proc_macro_attribute]
pub fn main(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  executor::main(item)
}

/// Package
///
/// A framework-like attribute placed in inline modules that creates all the mandatory elements
/// and optional elements related to `wtx::pkg::Package`.
///
/// ```rust
/// struct SomeApi;
///
/// #[wtx_macros::pkg(data_format(json_rpc("SomeEndpoint")), id(SomeApiId))]
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

/// Generates table fields separated by commas
#[proc_macro_derive(Table)]
pub fn table(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  match table::table(item) {
    Err(err) => syn::Error::from(err).to_compile_error().into(),
    Ok(elem) => elem,
  }
}

/// Allows the execution of asynchronous tests using the runtime provided by `WTX`.
#[proc_macro_attribute]
pub fn test(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  executor::test(item)
}
