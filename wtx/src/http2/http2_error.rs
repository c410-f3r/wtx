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
  /// The calling convention is not being respected. For example, in a client the method that reads
  /// data is being called before sending anything.
  BadLocalFlow,
  /// Number of active concurrent streams extrapolated the threshold
  ExceedAmountOfActiveConcurrentStreams,
  /// Frame has a zero stream ID but shouldn't because of its type.
  FrameIsZeroButShouldNot,
  /// The system only supports 2 header frames when sending data
  HeadersOverflow,
  /// Couldn't decode a header into a hpack buffer
  HpackDecodingBufferIsTooSmall,
  /// There are no bytes left to decode HPACK headers.
  InsufficientHpackBytes,
  #[doc = stream_id_must_not_be_zero!()]
  InvalidContinuationFrameZeroId,
  /// Length is greater than [`u32::MAX`].
  InvalidDataFrameDataLen,
  #[doc = stream_id_must_not_be_zero!()]
  InvalidDataFrameZeroId,
  /// Size updates of dynamic table can't be placed after the first header
  InvalidDynTableSizeUpdate,
  /// Frame pad was invalid bytes
  InvalidFramePad,
  #[doc = invalid_frame_bytes!()]
  InvalidGoAwayFrameBytes,
  #[doc = stream_id_must_be_zero!()]
  InvalidGoAwayFrameNonZeroId,
  /// A container does not contain an element referred by the given idx
  InvalidHpackIdx(Option<u32>),
  /// Header frame has mal formatted content
  InvalidHeaderData,
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
  LargeArbitraryFrameLen,
  /// Set of received data frames extrapolate delimited maximum length
  LargeBodyLen(Option<u32>, u32),
  /// Ignorable frames extrapolates delimited maximum length
  LargeIgnorableFrameLen,
  /// All trailer frames must include the EOS flag
  MissingEOSInTrailer,
  /// There are no buffers to create to new stream
  NoBuffersForNewStream,
  /// Counter-part did not return the correct bytes of a HTTP2 connection preface
  NoPreface,
  /// Frame size must be within 16384 and 16777215
  OutOfBoundsMaxFrameSize,
  /// Window size must be within 0 and 2147483647
  OutOfBoundsWindowSize,
  /// It is not possible to add trailers without data frames
  TrailersWithoutData,
  /// A stream frame was expected but instead a connection frame was received
  UnexpectedConnFrame,
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
  /// Type is out of range or unsupported.
  UnknownSettingFrameTy,
  /// A stream ID is not locally stored to allow the processing of data frames.
  UnknownDataStreamReceiver,
  /// A stream ID is not locally stored to allow the processing of header frames.
  UnknownHeaderStreamReceiver,
  /// A stream ID is not locally stored to allow the processing of reset frames.
  UnknownResetStreamReceiver,
  /// A stream ID is not locally stored to allow the processing of window update frames.
  UnknownWindowUpdateStreamReceiver,
  /// Length of a header name or value is limited to 127 bytes.
  UnsupportedHeaderNameOrValueLen,
  /// Push frames are deprecated and unsupported
  UnsupportedPushFrame,
  /// Server Push is deprecated and unsupported.
  UnsupportedServerPush,
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
  /// Windows size can not be reduced
  WindowSizeCanNotBeReduced,
}
