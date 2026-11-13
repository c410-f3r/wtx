/// Client API Framework Error
#[derive(Debug)]
pub enum ClientApiFrameworkError {
  /// No stored test response to return a result from a request
  TestTransportNoResponse,
}
