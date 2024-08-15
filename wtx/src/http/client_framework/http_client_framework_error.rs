/// Client Framework Error
#[derive(Debug)]
pub enum HttpClientFrameworkError {
  /// The remote server closed the connection
  ClosedConnection,
}
