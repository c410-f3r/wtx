#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamState {
  /// Final stage. Sent/received EOS_STREAM/RST_STREAM after
  /// [StreamState::HalfClosedLocal]/[StreamState::HalfClosedRemote] or sent/received
  /// RST_STREAM after [StreamState::Open].
  Closed,
  /// The system sent EOS_STREAM after [StreamState::Open].
  HalfClosedLocal,
  /// The system received EOS_STREAM after [StreamState::Open].
  HalfClosedRemote,
  /// Initial state. Awaiting initial headers.
  Idle,
  /// The system is receiving data after [StreamState::Open].
  Open,
}

impl StreamState {
  pub(crate) fn can_server_send(self) -> bool {
    matches!(self, Self::Open | Self::HalfClosedRemote)
  }
}
