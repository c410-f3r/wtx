//! Generic HTTP elements

mod abstract_headers;
#[cfg(feature = "http-client")]
mod client;
mod expected_header;
mod generic_header;
mod generic_request;
mod generic_response;
mod header_name;
mod headers;
mod method;
mod mime;
mod protocol;
mod request;
mod response;
mod response_data;
mod status_code;
mod version;

pub(crate) use abstract_headers::AbstractHeaders;
#[cfg(feature = "http-client")]
pub use client::Client;
pub use expected_header::ExpectedHeader;
pub use generic_header::GenericHeader;
pub use generic_request::GenericRequest;
pub use generic_response::GenericResponse;
pub use header_name::*;
pub use headers::Headers;
pub use method::Method;
pub use mime::Mime;
pub use protocol::Protocol;
pub use request::{Request, RequestMut, RequestRef};
pub use response::{Response, ResponseMut, ResponseRef};
pub use response_data::ResponseData;
pub use status_code::StatusCode;
pub use version::Version;

/// Maximum number of bytes for the name or the value of a header.
pub const MAX_HEADER_FIELD_LEN: usize = 256;
