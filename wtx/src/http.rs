//! Generic HTTP elements

mod abstract_headers;
mod generic_header;
mod generic_request;
mod generic_response;
mod header_name;
mod headers;
mod method;
mod mime;
mod protocol;
mod req_res_data;
mod request;
mod response;
pub mod server;
mod status_code;
mod version;

pub(crate) use abstract_headers::AbstractHeaders;
pub use generic_header::GenericHeader;
pub use generic_request::GenericRequest;
pub use generic_response::GenericResponse;
pub use header_name::*;
pub use headers::{Header, Headers};
pub use method::Method;
pub use mime::Mime;
pub use protocol::Protocol;
pub use req_res_data::ReqResData;
pub use request::{Request, RequestStr};
pub use response::Response;
pub use status_code::StatusCode;
pub use version::Version;

/// Maximum number of bytes for the name of a header.
pub const MAX_HEADER_NAME_LEN: usize = 128;
/// Maximum number of bytes for the value of a header.
pub const MAX_HEADER_VALUE_LEN: usize = 1024 + 256;

pub(crate) type _HeaderNameBuffer = crate::misc::ArrayVector<u8, MAX_HEADER_NAME_LEN>;
pub(crate) type _HeaderValueBuffer = crate::misc::ArrayVector<u8, MAX_HEADER_VALUE_LEN>;
