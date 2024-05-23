/// Status code used to indicate why an endpoint is closing the WebSocket connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseCode {
  /// Normal closure.
  Normal,
  /// An endpoint is not longer active.
  Away,
  /// Closing connection due to a protocol error.
  Protocol,
  /// An endpoint does not support a certain type of data.
  Unsupported,
  /// Closing frame without a status code.
  Status,
  /// Connection dropped without an error.
  Abnormal,
  /// Received data that differs from the frame type.
  Invalid,
  /// Generic error.
  Policy,
  /// Received a very large payload.
  Size,
  /// Client didn't receive extension from the server.
  Extension,
  /// An unexpected condition occurred.
  Error,
  /// Server is restarting.
  Restart,
  /// Server is busy and the client should reconnect.
  Again,
  /// MUST NOT be set as a status code in a close control frame by an endpoint. It is designated
  /// for use in applications expecting a status code to indicate that the connection was closed
  /// due to a failure to perform a TLS handshake
  Tls,
  /// Spaces without meaning reserved by the specification.
  Reserved(u16),
  /// IANA spaces reserved for use by libraries, frameworks, and applications.
  Iana(u16),
  /// Reserved for private use.
  Library(u16),
}

impl CloseCode {
  /// Checks if this instances is allowed.
  #[inline]
  pub fn is_allowed(self) -> bool {
    !matches!(self, Self::Reserved(_) | Self::Status | Self::Abnormal | Self::Tls)
  }
}

impl TryFrom<u16> for CloseCode {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u16) -> Result<Self, crate::Error> {
    Ok(match from {
      1000 => Self::Normal,
      1001 => Self::Away,
      1002 => Self::Protocol,
      1003 => Self::Unsupported,
      1005 => Self::Status,
      1006 => Self::Abnormal,
      1007 => Self::Invalid,
      1008 => Self::Policy,
      1009 => Self::Size,
      1010 => Self::Extension,
      1011 => Self::Error,
      1012 => Self::Restart,
      1013 => Self::Again,
      1015 => Self::Tls,
      1016..=2999 => Self::Reserved(from),
      3000..=3999 => Self::Iana(from),
      4000..=4999 => Self::Library(from),
      received => return Err(crate::Error::MISC_UnexpectedUint { received: received.into() }),
    })
  }
}

impl From<CloseCode> for u16 {
  #[inline]
  fn from(from: CloseCode) -> u16 {
    match from {
      CloseCode::Normal => 1000,
      CloseCode::Away => 1001,
      CloseCode::Protocol => 1002,
      CloseCode::Unsupported => 1003,
      CloseCode::Status => 1005,
      CloseCode::Abnormal => 1006,
      CloseCode::Invalid => 1007,
      CloseCode::Policy => 1008,
      CloseCode::Size => 1009,
      CloseCode::Extension => 1010,
      CloseCode::Error => 1011,
      CloseCode::Restart => 1012,
      CloseCode::Again => 1013,
      CloseCode::Tls => 1015,
      CloseCode::Iana(code) | CloseCode::Library(code) | CloseCode::Reserved(code) => code,
    }
  }
}
