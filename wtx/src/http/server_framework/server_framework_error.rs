/// Server Framework Error
#[derive(Debug)]
pub enum ServerFrameworkError {
  /// Client requested a CORS header that isn't allowed
  ForbiddenCorsHeader,
  /// Client requested a CORS method that isn't allowed
  ForbiddenCorsMethod,
  /// Client requested a CORS origin that isn't allowed
  ForbiddenCorsOrigin,
  /// Client sent a request with invalid WebSocket tunneling parameters
  InvalidWebSocketParameters,
  /// Entered in a route that has an incompatible operation mode
  OperationModeMismatch,
  /// Unknown path
  UnknownPath,
}
