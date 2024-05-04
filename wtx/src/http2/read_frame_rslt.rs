/// Read Frame Result
#[derive(Debug)]
pub enum ReadFrameRslt<T> {
  /// When a GOAWAY frame is sent or received.
  ClosedConnection,
  /// When a RST_STREAM frame is sent or received.
  ClosedStream,
  /// Remote part didn't send any frames
  IdleConnection,
  /// Resource was successfully fetched.
  Resource(T),
}

impl<T> ReadFrameRslt<T> {
  /// Extracts a successful resource, if any.
  #[inline]
  pub fn resource(self) -> Option<T> {
    if let Self::Resource(elem) = self {
      Some(elem)
    } else {
      None
    }
  }
}
