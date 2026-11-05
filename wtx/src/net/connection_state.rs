#[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
use {crate::sync::AtomicU8, core::sync::atomic::Ordering};

/// The state of a connection between two parties.
///
/// ```txt
///        +---> ReadClosed ---+
///        |                   |
///        +---> Draining -----+
///        |                   |
/// Open --+-------------------+-> ClosedAbruptly/ClosedGracefully
///        |                   |
///        +---> WriteClosed --+
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
  /// immediately flushed. In other words, [`Self::Open`] will jump directly to closed variants.
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
  pub const fn is_read_closed(self) -> bool {
    matches!(self, Self::ReadClosed)
  }

  /// Shortcut for [`ConnectionState::WriteClosed`].
  #[inline]
  pub const fn is_write_closed(self) -> bool {
    matches!(self, Self::WriteClosed)
  }

  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[inline]
  pub(crate) fn from_atomic(atomic: &AtomicU8) -> Self {
    Self::from_u8(atomic.load(Ordering::Relaxed))
  }

  #[allow(deprecated, reason = "`try_update` is not available in `portable-atomic`")]
  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[inline]
  pub(crate) fn atomic_update(atomic: &AtomicU8, cb: impl Fn(Self) -> Self) -> Self {
    match atomic.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |previous| {
      Some(cb(Self::from_u8(previous)).into())
    }) {
      Ok(prev) | Err(prev) => Self::from_u8(prev),
    }
  }

  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[inline]
  pub(crate) const fn on_abrupt_close(self) -> Self {
    match self {
      Self::ClosedGracefully => Self::ClosedGracefully,
      Self::ClosedAbruptly | Self::Draining | Self::Open | Self::ReadClosed | Self::WriteClosed => {
        Self::ClosedAbruptly
      }
    }
  }

  /// Used when a operation observes a remote close and also sends a close notification.
  ///
  /// Both directions will be closed so the connection will be either gracefully closed or remains
  /// abruptly closed if that was already the case.
  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[inline]
  pub(crate) const fn on_both_close(self) -> Self {
    self.on_remote_close().on_local_close()
  }

  #[cfg(feature = "http2")]
  #[inline]
  pub(crate) const fn on_draining(self) -> Self {
    match self {
      Self::Open => Self::Draining,
      Self::ClosedAbruptly
      | Self::ClosedGracefully
      | Self::Draining
      | Self::ReadClosed
      | Self::WriteClosed => self,
    }
  }

  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[inline]
  pub(crate) const fn on_local_close(self) -> Self {
    match self {
      Self::Draining | Self::Open | Self::WriteClosed => Self::WriteClosed,
      Self::ClosedGracefully | Self::ReadClosed => Self::ClosedGracefully,
      Self::ClosedAbruptly => Self::ClosedAbruptly,
    }
  }

  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[inline]
  pub(crate) const fn on_remote_close(self) -> Self {
    match self {
      Self::Draining | Self::Open | Self::ReadClosed => Self::ReadClosed,
      Self::ClosedGracefully | Self::WriteClosed => Self::ClosedGracefully,
      Self::ClosedAbruptly => Self::ClosedAbruptly,
    }
  }

  #[cfg(any(feature = "http2", feature = "tls", feature = "web-socket"))]
  #[expect(clippy::match_same_arms, reason = "more readable")]
  #[inline]
  const fn from_u8(value: u8) -> Self {
    match value {
      0 => Self::ClosedAbruptly,
      1 => Self::ClosedGracefully,
      2 => Self::Draining,
      3 => Self::Open,
      4 => Self::ReadClosed,
      5 => Self::WriteClosed,
      // Basically impossible because it is an internal method
      _ => Self::ClosedAbruptly,
    }
  }
}

impl TryFrom<u8> for ConnectionState {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> Result<Self, Self::Error> {
    Ok(match from {
      0 => ConnectionState::ClosedAbruptly,
      1 => ConnectionState::ClosedGracefully,
      2 => ConnectionState::Draining,
      3 => ConnectionState::Open,
      4 => ConnectionState::ReadClosed,
      5 => ConnectionState::WriteClosed,
      _ => return Err(crate::Error::InvalidConnectionState(from)),
    })
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
