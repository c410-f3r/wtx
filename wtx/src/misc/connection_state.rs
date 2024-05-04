/// The state of a connection between two parties.
#[derive(Clone, Copy, Debug)]
pub enum ConnectionState {
  /// Is locally open. Does not necessary means that both parties are in the same state.
  Open,
  /// Is locally closed. Does not necessary means that both parties are in the same state.
  Closed,
}

impl ConnectionState {
  /// Shortcut for [ConnectionState::Closed].
  #[inline]
  pub fn is_closed(self) -> bool {
    matches!(self, Self::Closed)
  }

  /// Shortcut for [ConnectionState::Open].
  #[inline]
  pub fn is_open(self) -> bool {
    matches!(self, Self::Open)
  }
}

impl From<bool> for ConnectionState {
  #[inline]
  fn from(from: bool) -> Self {
    if from {
      Self::Open
    } else {
      Self::Closed
    }
  }
}
