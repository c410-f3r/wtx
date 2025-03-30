//! Generic HTTP elements

#[cfg(feature = "http-client-pool")]
pub mod client_pool;
#[cfg(any(feature = "http-client-pool", feature = "http-server-framework"))]
mod conn_params;
#[cfg(feature = "http-cookie")]
mod cookie;
mod generic_header;
mod generic_request;
mod generic_response;
mod header_name;
mod headers;
mod http_client;
mod http_error;
mod method;
mod mime;
mod misc;
mod operation_mode;
#[cfg(feature = "nightly")]
mod optioned_server;
mod protocol;
mod req_res_buffer;
mod req_res_builder;
mod req_res_data;
mod request;
mod response;
#[cfg(feature = "http-server-framework")]
pub mod server_framework;
#[cfg(feature = "http-session")]
mod session;
mod status_code;
mod version;

#[cfg(feature = "http-session")]
pub use cookie::*;
pub use generic_header::GenericHeader;
pub use generic_request::GenericRequest;
pub use generic_response::GenericResponse;
pub use header_name::*;
pub use headers::{Header, Headers, Trailers};
pub use http_client::HttpClient;
pub use http_error::HttpError;
pub use method::Method;
pub use mime::Mime;
pub use misc::*;
pub use operation_mode::*;
#[cfg(feature = "nightly")]
pub use optioned_server::OptionedServer;
pub use protocol::Protocol;
pub use req_res_buffer::ReqResBuffer;
pub use req_res_builder::*;
pub use req_res_data::{ReqResData, ReqResDataMut};
pub use request::Request;
pub use response::Response;
#[cfg(feature = "http-session")]
pub use session::*;
pub use status_code::StatusCode;
pub use version::Version;

/// Maximum number of bytes for the name of a header.
pub const MAX_HEADER_NAME_LEN: usize = 128;
/// Maximum number of bytes for the value of a header.
pub const MAX_HEADER_VALUE_LEN: usize = 1024 * 3;
/// Current user agent of this project
pub const WTX_USER_AGENT: &str = concat!("wtx/", env!("CARGO_PKG_VERSION"));

pub(crate) const _MAX_AUTHORITY_LEN: usize = 64;
pub(crate) const _MAX_PATH_LEN: usize = 128;
pub(crate) const _MAX_SCHEME_LEN: usize = 16;

pub(crate) type _HeaderNameBuffer = crate::misc::ArrayVector<u8, MAX_HEADER_NAME_LEN>;
pub(crate) type _HeaderValueBuffer = crate::misc::ArrayVector<u8, MAX_HEADER_VALUE_LEN>;
