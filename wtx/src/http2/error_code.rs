create_enum! {
  /// HTTP/2 error codes.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum ErrorCode<u32> {
    /// The associated condition is not a result of an error.
    NoError = (0),
    /// The endpoint detected an unspecific protocol error.
    ProtocolError = (1),
    /// The endpoint encountered an unexpected internal error.
    InternalError = (2),
    /// The endpoint detected that its peer violated the flow-control protocol.
    FlowControlError = (3),
    /// The endpoint sent a SETTINGS frame but did not receive a response in
    /// a timely manner.
    SettingsTimeout = (4),
    /// The endpoint received a frame after a stream was half-closed.
    StreamClosed = (5),
    /// The endpoint received a frame with an invalid size.
    FrameSizeError = (6),
    /// The endpoint refused the stream prior to performing any application
    /// processing.
    RefusedStream = (7),
    /// Used by the endpoint to indicate that the stream is no longer needed.
    Cancel = (8),
    /// The endpoint is unable to maintain the header compression context for
    /// the connection.
    CompressionError = (9),
    /// The connection established in response to a CONNECT request was reset
    /// or abnormally closed.
    ConnectError = (10),
    /// The endpoint detected that its peer is exhibiting a behavior that might
    /// be generating excessive load.
    EnhanceYourCalm = (11),
    /// The underlying transport has properties that do not meet minimum
    /// security requirements.
    InadequateSecurity = (12),
    /// The endpoint requires that HTTP/1.1 be used instead of HTTP/2.
    Http11Requires = (13),
  }
}
