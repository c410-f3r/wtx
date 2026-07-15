/// The state of a connection between two parties.
///
/// ```txt
///         +---> ReadClosed ---+
///         |                   |
///         +----> Draining ----+
///         |                   |
/// Open --+-------------------+-> ClosedAbruptly/ClosedGracefully
///         |                   |
///         +---> WriteClosed --+
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConnectionState {
  /// Is closed for both reads and writes. The connection was abruptly closed.
  ClosedAbruptly,
  /// Is closed for both reads and writes. The connection has gracefully closed.
  ClosedGracefully,
  /// Signals the intention to transit to closed, however, some internal state still
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
  /// immediately flushed. In other words, [`Self::Open`] will jump directly to closed.
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
    matches!(self, Self::ClosedAbruptly | Self::ClosedGracefully | Self::ReadClosed)
  }

  /// Returns `true` if the connection is no longer readable or writable.
  #[inline]
  pub const fn cannot_read_or_write(self) -> bool {
    matches!(
      self,
      Self::ClosedAbruptly | Self::ClosedGracefully | Self::ReadClosed | Self::WriteClosed
    )
  }

  /// Returns `true` if the connection is no longer writable.
  #[inline]
  pub const fn cannot_write(self) -> bool {
    matches!(self, Self::ClosedAbruptly | Self::ClosedGracefully | Self::WriteClosed)
  }

  /// Shortcut for [`ConnectionState::ClosedAbruptly`] | [`ConnectionState::ClosedGracefully`].
  #[inline]
  pub const fn is_closed(self) -> bool {
    matches!(self, Self::ClosedAbruptly | Self::ClosedGracefully)
  }

  /// Shortcut for [`ConnectionState::Draining`].
  #[inline]
  pub const fn is_draining(self) -> bool {
    matches!(self, Self::Draining)
  }

  /// Returns `true` if the state is [`ConnectionState::ClosedAbruptly`],
  /// [`ConnectionState::ClosedGracefully`] or  [`ConnectionState::Draining`].
  #[inline]
  pub const fn is_full_close(self) -> bool {
    matches!(self, Self::ClosedAbruptly | Self::ClosedGracefully | Self::Draining)
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
      0 => ConnectionState::ClosedAbruptly,
      1 => ConnectionState::ClosedGracefully,
      2 => ConnectionState::Draining,
      3 => ConnectionState::Open,
      4 => ConnectionState::ReadClosed,
      _ => ConnectionState::WriteClosed,
    }
  }
}

impl From<ConnectionState> for u8 {
  #[inline]
  fn from(from: ConnectionState) -> Self {
    match from {
      ConnectionState::ClosedAbruptly => 0,
      ConnectionState::ClosedGracefully => 1,
      ConnectionState::Draining => 2,
      ConnectionState::Open => 3,
      ConnectionState::ReadClosed => 4,
      ConnectionState::WriteClosed => 5,
    }
  }
}
