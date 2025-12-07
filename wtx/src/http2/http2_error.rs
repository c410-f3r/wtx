macro_rules! invalid_frame_bytes {
  () => {
    "Received bytes don't have an expected format or stream ID is zero."
  };
}

macro_rules! stream_id_must_be_zero {
  () => {
    "Stream ID must be zero"
  };
}

macro_rules! stream_id_must_not_be_zero {
  () => {
    "Stream ID must not be zero"
  };
}

/// Errors for `Http2`.
#[derive(Debug)]
pub enum Http2Error {
  /// The number of opened streams extrapolated the threshold
  ExceedAmountOfOpenedStreams,
  /// The number of active concurrent streams extrapolated the threshold
  ExceedAmountOfActiveConcurrentStreams,
  /// The system only supports 2 header frames when sending data
  HeadersOverflow,
  /// Couldn't decode a header into a hpack buffer
  HpackDecodingBufferIsTooSmall,
  /// A header was not fully constructed.
  IncompleteHeader,
  /// There are no bytes left to decode HPACK headers.
  InsufficientHpackBytes,
  /// Received data that should be exclusive for servers
  InvalidClientHeader,
  /// Content length has a value that is different than the actual payload length
  InvalidContentLength,
  /// Continuation frame found in invalid order
  InvalidContinuationFrame,
  /// Length is greater than [`u32::MAX`].
  InvalidDataFrameDataLen,
  #[doc = stream_id_must_not_be_zero!()]
  InvalidDataFrameZeroId,
  /// Size updates of dynamic table can't be placed after the first header
  InvalidDynTableSizeUpdate,
  /// Provided number does not match any HTTP/2 error code
  InvalidErrorCode,
  /// Frame pad was invalid bytes
  InvalidFramePad,
  #[doc = invalid_frame_bytes!()]
  InvalidGoAwayFrameBytes,
  #[doc = stream_id_must_be_zero!()]
  InvalidGoAwayFrameNonZeroId,
  /// A container does not contain an element referred by the given idx
  InvalidHpackIdx(u32),
  /// Header frame has mal-formatted content
  InvalidHeaderFrame,
  #[doc = stream_id_must_not_be_zero!()]
  InvalidHeadersFrameZeroId,
  #[doc = invalid_frame_bytes!()]
  InvalidPingFrameBytes,
  #[doc = stream_id_must_be_zero!()]
  InvalidPingFrameNonZeroId,
  /// Invalid frame after received EOS
  InvalidReceivedFrameAfterEos,
  #[doc = invalid_frame_bytes!()]
  InvalidResetStreamFrameBytes,
  #[doc = stream_id_must_not_be_zero!()]
  InvalidResetStreamFrameZeroId,
  /// Stream is in a state that forbids sending more data
  InvalidSendStreamState,
  /// Received insufficient data to fullfil a server's header
  InvalidServerHeader,
  /// URI is too large for servers
  InvalidServerHeaderUriOverflow,
  /// Settings frames length must be divisible  by 6
  InvalidSettingsFrameLength,
  #[doc = stream_id_must_be_zero!()]
  InvalidSettingsFrameNonZeroId,
  /// Settings ACK must be empty
  InvalidSettingsFrameNonEmptyAck,
  #[doc = invalid_frame_bytes!()]
  InvalidWindowUpdateFrameBytes,
  /// Size increment can't be greater than`2^31 - 1`.
  InvalidWindowUpdateSize,
  /// Size increment must be greater than zero
  InvalidWindowUpdateZeroIncrement,
  /// Received arbitrary frame extrapolates delimited maximum length
  LargeArbitraryFrameLen {
    /// Received length from a response
    received: u32,
  },
  /// Set of received data frames extrapolate delimited maximum length
  LargeBodyLen(Option<u32>),
  /// Ignorable frames extrapolates delimited maximum length
  LargeIgnorableFrameLen,
  /// All trailer frames must include the EOS flag
  MissingEOSInTrailer,
  /// Counter-part did not return the correct bytes of a HTTP2 connection preface
  NoPreface,
  /// Received index is greater than the supported range
  OutOfBoundsIndex,
  /// Frame size must be within 16384 and 16777215
  OutOfBoundsMaxFrameSize,
  /// Window size must be within 0 and 2147483647
  OutOfBoundsWindowSize,
  /// Browsers don't support PUSH_PROMISE
  PushPromiseIsUnsupported,
  /// Received frame should be a continuation frame with correct ID
  UnexpectedContinuationFrame,
  /// Decoding logic encountered an unexpected ending string signal.
  UnexpectedEndingHuffman,
  /// Header frames must be received only once per block
  UnexpectedHeaderFrame,
  /// Received an Hpack index that does not adhere to the standard
  UnexpectedHpackIdx,
  /// The stream is in a state where it can only receive control frames
  UnexpectedNonControlFrame,
  /// Unknown header name.
  UnexpectedPreFixedHeaderName,
  /// Servers must only receive odd IDs or IDs are lower than the current highest value
  UnexpectedStreamId,
  /// A stream ID is not locally stored to allow the processing of data frames.
  UnknownDataStreamReceiver,
  /// A programming error that shouldn't never happen
  UnknownInitialServerHeaderId(u32),
  /// A stream ID is not locally stored to allow the processing of reset frames.
  UnknownResetStreamReceiver,
  /// Type is out of range or unsupported.
  UnknownSettingFrameTy,
  /// Stream id doesn't exist locally
  UnknownStreamId,
  /// A stream ID is not locally stored.
  UnknownStreamReceiver,
  /// A stream ID is not locally stored to allow the processing of window update frames.
  UnknownWindowUpdateStreamReceiver,
  /// Length of a header name or value is limited to 127 bytes.
  UnsupportedHeaderNameOrValueLen,
  #[doc = concat!(
    "The system does not support more than",
    _max_continuation_frames!(),
    " continuation frames."
  )]
  VeryLargeAmountOfContinuationFrames,
  #[doc = concat!(
    "The system does not support more than",
    _max_frames_mismatches!(),
    " fetches of frames with mismatches IDs or mismatches types"
  )]
  VeryLargeAmountOfFrameMismatches,
  /// Header integers must be equal or lesser than `u16::MAX`
  VeryLargeHeaderInteger,
  /// Received headers is too large to sent
  VeryLargeHeadersLen,
  /// Continuation frames are not supported in HTTP/2
  WebSocketContinuationFrame,
}
