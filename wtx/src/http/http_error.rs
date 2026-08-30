use crate::http::{KnownHeaderName, Method, Mime};

/// Http error
#[derive(Clone, Copy, Debug)]
pub enum HttpError {
  /// Generic request error
  BadRequest,
  /// Invalid `form/data` content
  InvalidFormDataContent,
  /// Invalid HTTP/2 or HTTP/3 header
  InvalidHttp2pContent,
  /// Header names can not have more than 64 bytes
  LargeHeaderName,
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
  /// `TlsConfig` is mandatory for TLS connections
  TlsConnectionRequireTlsConfig,
  /// Content-Type mismatch
  UnexpectedContentType {
    /// Expected method
    expected: Mime,
  },
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
  /// Unknown `WebAuthn` Algorithm
  UnknownWebAuthnAlg,
  /// URI mismatch
  UriMismatch,
}
