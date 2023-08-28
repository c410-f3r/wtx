/// Errors related to the WebSocket module
#[derive(Debug)]
pub enum WebSocketError {
    /// Received close frame has invalid parameters.
    InvalidCloseFrame,
    /// Header indices are out-of-bounds or the number of bytes are too small.
    InvalidFrameHeaderBounds,
    /// No op code can be represented with the provided byte.
    InvalidOpCodeByte {
        /// Provided byte
        provided: u8,
    },
    /// Payload indices are out-of-bounds or the number of bytes are too small.
    InvalidPayloadBounds,

    /// Server received a frame without a mask.
    MissingFrameMask,
    /// Status code is expected to be
    MissingSwitchingProtocols,

    /// Received control frame wasn't supposed to be fragmented.
    UnexpectedFragmentedControlFrame,
    /// The first frame of a message is a continuation or the following frames are not a
    /// continuation.
    UnexpectedMessageFrame,

    /// It it not possible to read a frame of a connection that was previously closed.
    ConnectionClosed,
    /// Reserved bits are not zero.
    ReservedBitsAreNotZero,
    /// Control frames have a maximum allowed size.
    VeryLargeControlFrame,
    /// Frame payload exceeds the defined threshold.
    VeryLargePayload,
}
