/// The state of a connection between two parties.
///
/// ```txt
///        +---> ReadClosed ---+
///        |                   |
/// Open --+-------------------+-> Closed
///        |                   |
///        +---> WriteClosed --+
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConnectionState {
  /// Is closed for both reads and writes.
  Closed,
  /// Is open for both reads and writes.
  Open,
  /// Is closed for reading.
  ///
  /// Happens when the desire to end an connection was initialized by the remote peer. You
  /// shouldn't set this state based on local actions.
  ///
  /// In sequential code this state will never occur because any remaining state will be
  /// immediately flushed. In other words, [`Self::Open`] will jump directly to [`Self::Closed`].
  ///
  /// In protocol like HTTP/2 it is possible to briefly read external data, nevertheless, this
  /// state stills signals the transition for [`Self::Closed`].
  ReadClosed,
  /// Is closed for writing.
  ///
  /// Happens when the desire to end an connection was initiated locally. You shouldn't set
  /// this state based on remote actions.
  ///
  /// In protocol like HTTP/2 it is possible to briefly send local data, nevertheless, this
  /// state stills signals the transition for [`Self::Closed`].
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
      1 => ConnectionState::Open,
      2 => ConnectionState::ReadClosed,
      _ => ConnectionState::WriteClosed,
    }
  }
}

impl From<ConnectionState> for u8 {
  #[inline]
  fn from(from: ConnectionState) -> Self {
    match from {
      ConnectionState::Closed => 0,
      ConnectionState::Open => 1,
      ConnectionState::ReadClosed => 2,
      ConnectionState::WriteClosed => 3,
    }
  }
}
