/// Client API Framework Error
#[derive(Debug)]
pub enum ClientApiFrameworkError {
  /// The server closed the connection
  ClosedWsConnection,
  /// No stored test response to return a result from a request
  TestTransportNoResponse,
}
