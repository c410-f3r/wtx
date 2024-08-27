//! Generic HTTP elements

mod abstract_headers;
#[cfg(feature = "http-client-framework")]
pub mod client_framework;
#[cfg(any(feature = "http-client-framework", feature = "http-server-framework"))]
mod conn_params;
mod generic_header;
mod generic_request;
mod generic_response;
mod header_name;
mod headers;
mod http_error;
mod low_level_server;
mod method;
mod mime;
mod protocol;
mod req_res_buffer;
mod req_res_data;
mod req_uri;
mod request;
mod response;
#[cfg(feature = "http-server-framework")]
pub mod server_framework;
mod status_code;
mod version;

pub(crate) use abstract_headers::AbstractHeaders;
#[cfg(any(feature = "http-client-framework", feature = "http-server-framework"))]
pub(crate) use conn_params::ConnParams;
pub use generic_header::GenericHeader;
pub use generic_request::GenericRequest;
pub use generic_response::GenericResponse;
pub use header_name::*;
pub use headers::{Header, Headers};
pub use http_error::HttpError;
pub use low_level_server::LowLevelServer;
pub use method::Method;
pub use mime::Mime;
pub use protocol::Protocol;
pub use req_res_buffer::ReqResBuffer;
pub use req_res_data::{ReqResData, ReqResDataMut};
pub use req_uri::ReqUri;
pub use request::Request;
pub use response::Response;
pub use status_code::StatusCode;
pub use version::Version;

pub(crate) const _MAX_AUTHORITY_LEN: usize = 64;
pub(crate) const _MAX_PATH_LEN: usize = 128;
pub(crate) const _MAX_SCHEME_LEN: usize = 16;

/// Maximum number of bytes for the name of a header.
pub const MAX_HEADER_NAME_LEN: usize = 128;
/// Maximum number of bytes for the value of a header.
pub const MAX_HEADER_VALUE_LEN: usize = 1024 + 1024;

pub(crate) type _HeaderNameBuffer = crate::misc::ArrayVector<u8, MAX_HEADER_NAME_LEN>;
pub(crate) type _HeaderValueBuffer = crate::misc::ArrayVector<u8, MAX_HEADER_VALUE_LEN>;
