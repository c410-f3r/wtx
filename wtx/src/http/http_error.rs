use crate::http::{KnownHeaderName, Method};

/// Http error
#[derive(Debug)]
pub enum HttpError {
  /// Generic request error
  BadRequest,
  /// Invalid HTTP/2 or HTTP/3 header
  InvalidHttp2pContent,
  /// Missing Header
  MissingHeader(
    /// Expected header name
    KnownHeaderName,
  ),
  /// Received request does not contain a method field
  MissingRequestMethod,
  /// Received response does not contain a status code field
  MissingResponseStatusCode,
  /// The URI doesn't have any placeholder
  MissingUriPlaceholder,
  /// Content-Type mismatch
  UnexpectedContentType,
  /// HTTP version does not match the expected method.
  UnexpectedHttpMethod {
    /// Expected method
    expected: Method,
  },
  /// Unknown header name.
  UnknownHeaderNameFromBytes {
    /// Received length
    length: u32,
  },
  /// URI mismatch
  UriMismatch,
}
