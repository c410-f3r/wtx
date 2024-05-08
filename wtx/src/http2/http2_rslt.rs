/// HTTP/2 Result
#[derive(Debug)]
pub enum Http2Rslt<T> {
  /// When a GOAWAY frame is sent or received.
  ClosedConnection,
  /// When a RST_STREAM frame is sent or received.
  ClosedStream,
  /// Resource was successfully fetched.
  Resource(T),
}

impl<T> Http2Rslt<T> {
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

/// HTTP/2 Result - Extended
#[derive(Debug)]
pub(crate) enum Http2RsltExt<T> {
  ClosedConnection,
  ClosedStream,
  Resource(T),
  // Read: Stream is not returning new data
  // Write: Frame is awaiting an WINDOW_UPDATE frame
  Idle,
}
