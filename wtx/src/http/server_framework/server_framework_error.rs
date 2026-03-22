/// Server Framework Error
#[derive(Debug)]
pub enum ServerFrameworkError {
  /// Client sent a value in `Access-Control-Request-Headers` that isn't locally allowed.
  ForbiddenCorsHeader,
  /// Client sent a value in `Access-Control-Request-Method` that isn't locally allowed.
  ForbiddenCorsMethod,
  /// Client sent an origin that isn't locally allowed.
  ForbiddenCorsOrigin,
  /// The spec states that `Access-Control-Allow-Origin: *`, `Access-Control-Allow-Headers: *`,
  /// `Access-Control-Allow-Methods: *`, and `Access-Control-Expose-Headers: *` are not valid when
  /// credentials are involved.
  ForbiddenLocalCorsParameters,
  /// Client sent a request with invalid WebSocket tunneling parameters
  InvalidWebSocketParameters,
  /// Entered in a route that has an incompatible operation mode
  OperationModeMismatch,
  /// Unknown path
  UnknownPath,
}
