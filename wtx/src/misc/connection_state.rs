use crate::misc::{Lease, LeaseMut};

/// The state of a connection between two parties.
#[derive(Clone, Copy, Debug)]
pub enum ConnectionState {
  /// Is locally closed. Does not necessary means that both parties are in the same state.
  Closed,
  /// Is locally open. Does not necessary means that both parties are in the same state.
  Open,
}

impl ConnectionState {
  /// Shortcut for [`ConnectionState::Closed`].
  #[inline]
  pub fn is_closed(self) -> bool {
    matches!(self, Self::Closed)
  }

  /// Shortcut for [`ConnectionState::Open`].
  #[inline]
  pub fn is_open(self) -> bool {
    matches!(self, Self::Open)
  }
}

impl Lease<ConnectionState> for ConnectionState {
  #[inline]
  fn lease(&self) -> &ConnectionState {
    self
  }
}

impl LeaseMut<ConnectionState> for ConnectionState {
  #[inline]
  fn lease_mut(&mut self) -> &mut ConnectionState {
    self
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

impl From<ConnectionState> for bool {
  #[inline]
  fn from(from: ConnectionState) -> Self {
    match from {
      ConnectionState::Closed => false,
      ConnectionState::Open => true,
    }
  }
}
