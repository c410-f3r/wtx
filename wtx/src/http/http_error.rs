use crate::http::{KnownHeaderName, Method};

/// Http error
#[derive(Debug)]
pub enum HttpError {
  /// Client requrested a CORS header that isn't allowed
  ForbiddenCorsHeader,
  /// Client requrested a CORS method that isn't allowed
  ForbiddenCorsMethod,
  /// Client requrested a CORS origin that isn't allowed
  ForbiddenCorsOrigin,
  /// The length of a header field must be within a threshold.
  HeaderFieldIsTooLarge,
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
    length: usize,
  },
  /// URI mismatch
  UriMismatch,
}
