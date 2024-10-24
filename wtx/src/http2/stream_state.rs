#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum StreamState {
  /// Final stage. Sent/received `END_STREAM`/`RST_STREAM` after
  /// [`StreamState::HalfClosedLocal`]/[`StreamState::HalfClosedRemote`] or sent/received
  /// `RST_STREAM` after [`StreamState::Open`].
  Closed,
  /// The system sent `END_STREAM` after [`StreamState::Open`].
  HalfClosedLocal,
  /// The system received `END_STREAM` after [`StreamState::Open`].
  HalfClosedRemote,
  /// Initial state. Awaiting initial headers.
  Idle,
  /// The system is receiving data after [`StreamState::Open`].
  Open,
}

impl StreamState {
  /// If the system can send to a peer regardless of the frame type.
  pub(crate) fn can_send<const IS_CLIENT: bool>(self) -> bool {
    if IS_CLIENT {
      matches!(self, Self::Idle | Self::Open)
    } else {
      matches!(self, Self::HalfClosedRemote | Self::Open)
    }
  }

  /// Received End Of Stream
  ///
  /// If the receiving part received an EOS from a peer.
  pub(crate) fn recv_eos(self) -> bool {
    matches!(self, Self::HalfClosedRemote | Self::Closed)
  }
}
