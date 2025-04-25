/// WebSocket Error
#[derive(Debug)]
pub enum WebSocketError {
  /// HTTP headers must be unique.
  DuplicatedHeader,
  /// Received close frame has invalid parameters.
  InvalidCloseFrame,
  /// Received an invalid header compression parameter.
  InvalidCompressionHeaderParameter,
  /// The client sent an invalid mask bit.
  InvalidMaskBit,
  /// Server received a frame without a mask.
  MissingFrameMask,
  /// Handshake response should return a 101 code.
  MissingSwitchingProtocols {
    /// The actual response code received
    found: Option<u16>,
  },
  /// Received control frame wasn't supposed to be fragmented.
  UnexpectedFragmentedControlFrame,
  /// For example, the first frame of a message is a continuation.
  UnexpectedFrame,
  /// Control frames have a maximum allowed size.
  VeryLargeControlFrame,
  /// Frame payload exceeds the defined threshold.
  VeryLargePayload,
}
