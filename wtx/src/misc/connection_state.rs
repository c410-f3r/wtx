/// The state of a connection between two parties.
///
/// ```txt
///         +---> ReadClosed ---+
///         |                   |
/// Open --+-------------------+-> Closed
///         |                   |
///         +---> Terminating --+
///         |                   |
///         +---> WriteClosed --+
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConnectionState {
  /// Is closed for both reads and writes.
  Closed,
  /// Signals the intention to transit to [`Self::Closed`], however, some internal state still
  /// needs to be managed through a few more receipts and sends.
  Draining,
  /// Is open for both reads and writes.
  Open,
  /// Is closed for reading.
  ///
  /// Happens when the desire to end an connection was initialized by the remote peer. You
  /// shouldn't set this state based on local actions.
  ///
  /// In sequential code this state will never occur because any remaining state will be
  /// immediately flushed. In other words, [`Self::Open`] will jump directly to [`Self::Closed`].
  ReadClosed,
  /// Is closed for writing.
  ///
  /// Happens when the desire to end an connection was initiated locally. You shouldn't set
  /// this state based on remote actions.
  WriteClosed,
}

impl ConnectionState {
  /// Returns `true` if the connection is no longer readable.
  #[inline]
  pub const fn cannot_read(self) -> bool {
    matches!(self, Self::Closed | Self::ReadClosed)
  }

  /// Returns `true` if the connection is no longer writable.
  #[inline]
  pub const fn cannot_write(self) -> bool {
    matches!(self, Self::Closed | Self::WriteClosed)
  }

  /// Shortcut for [`ConnectionState::Closed`].
  #[inline]
  pub const fn is_closed(self) -> bool {
    matches!(self, Self::Closed)
  }

  /// Shortcut for [`ConnectionState::Draining`].
  #[inline]
  pub const fn is_draining(self) -> bool {
    matches!(self, Self::Draining)
  }

  /// Returns `true` if the state is [`ConnectionState::Closed`] or  [`ConnectionState::Draining`].
  #[inline]
  pub const fn is_full_close(self) -> bool {
    matches!(self, Self::Closed | Self::Draining)
  }

  /// Shortcut for [`ConnectionState::Open`].
  #[inline]
  pub const fn is_open(self) -> bool {
    matches!(self, Self::Open)
  }

  /// Shortcut for [`ConnectionState::ReadClosed`].
  #[inline]
  pub const fn is_read_closed(&self) -> bool {
    matches!(self, Self::ReadClosed)
  }

  /// Shortcut for [`ConnectionState::WriteClosed`].
  #[inline]
  pub const fn is_write_closed(&self) -> bool {
    matches!(self, Self::WriteClosed)
  }
}

impl From<u8> for ConnectionState {
  #[inline]
  fn from(from: u8) -> Self {
    match from {
      0 => ConnectionState::Closed,
      1 => ConnectionState::Draining,
      2 => ConnectionState::Open,
      3 => ConnectionState::ReadClosed,
      _ => ConnectionState::WriteClosed,
    }
  }
}

impl From<ConnectionState> for u8 {
  #[inline]
  fn from(from: ConnectionState) -> Self {
    match from {
      ConnectionState::Closed => 0,
      ConnectionState::Draining => 1,
      ConnectionState::Open => 2,
      ConnectionState::ReadClosed => 3,
      ConnectionState::WriteClosed => 4,
    }
  }
}
