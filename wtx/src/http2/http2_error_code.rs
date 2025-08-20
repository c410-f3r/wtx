use crate::http2::{Http2Error, misc::protocol_err};

/// HTTP/2 error codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Http2ErrorCode {
  /// The associated condition is not a result of an error.
  NoError,
  /// The endpoint detected an unspecific protocol error.
  ProtocolError,
  /// The endpoint encountered an unexpected internal error.
  InternalError,
  /// The endpoint detected that its peer violated the flow-control protocol.
  FlowControlError,
  /// The endpoint sent a SETTINGS frame but did not receive a response in
  /// a timely manner.
  SettingsTimeout,
  /// The endpoint received a frame after a stream was half-closed.
  StreamClosed,
  /// The endpoint received a frame with an invalid size.
  FrameSizeError,
  /// The endpoint refused the stream prior to performing any application
  /// processing.
  RefusedStream,
  /// Used by the endpoint to indicate that the stream is no longer needed.
  Cancel,
  /// The endpoint is unable to maintain the header compression context for
  /// the connection.
  CompressionError,
  /// The connection established in response to a CONNECT request was reset
  /// or abnormally closed.
  ConnectError,
  /// The endpoint detected that its peer is exhibiting a behavior that might
  /// be generating excessive load.
  EnhanceYourCalm,
  /// The underlying transport has properties that do not meet minimum
  /// security requirements.
  InadequateSecurity,
  /// The endpoint requires HTTP/1.1 instead of HTTP/2.
  Http11Requires,
}

impl TryFrom<u32> for Http2ErrorCode {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u32) -> Result<Self, Self::Error> {
    Ok(match value {
      0 => Http2ErrorCode::NoError,
      1 => Http2ErrorCode::ProtocolError,
      2 => Http2ErrorCode::InternalError,
      3 => Http2ErrorCode::FlowControlError,
      4 => Http2ErrorCode::SettingsTimeout,
      5 => Http2ErrorCode::StreamClosed,
      6 => Http2ErrorCode::FrameSizeError,
      7 => Http2ErrorCode::RefusedStream,
      8 => Http2ErrorCode::Cancel,
      9 => Http2ErrorCode::CompressionError,
      10 => Http2ErrorCode::ConnectError,
      11 => Http2ErrorCode::EnhanceYourCalm,
      12 => Http2ErrorCode::InadequateSecurity,
      13 => Http2ErrorCode::Http11Requires,
      _ => return Err(protocol_err(Http2Error::InvalidErrorCode)),
    })
  }
}

impl From<Http2ErrorCode> for u32 {
  #[inline]
  fn from(value: Http2ErrorCode) -> Self {
    match value {
      Http2ErrorCode::NoError => 0,
      Http2ErrorCode::ProtocolError => 1,
      Http2ErrorCode::InternalError => 2,
      Http2ErrorCode::FlowControlError => 3,
      Http2ErrorCode::SettingsTimeout => 4,
      Http2ErrorCode::StreamClosed => 5,
      Http2ErrorCode::FrameSizeError => 6,
      Http2ErrorCode::RefusedStream => 7,
      Http2ErrorCode::Cancel => 8,
      Http2ErrorCode::CompressionError => 9,
      Http2ErrorCode::ConnectError => 10,
      Http2ErrorCode::EnhanceYourCalm => 11,
      Http2ErrorCode::InadequateSecurity => 12,
      Http2ErrorCode::Http11Requires => 13,
    }
  }
}
