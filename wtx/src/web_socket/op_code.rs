create_enum! {
  /// Defines how to interpret the payload data.
  #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum OpCode<u8> {
    /// Continuation of a previous frame.
    Continuation = (0b0000_0000),
    /// UTF-8 text.
    Text = (0b0000_0001),
    /// Opaque bytes.
    Binary = (0b0000_0010),
    /// Connection is closed.
    Close = (0b0000_1000),
    /// Test reachability.
    Ping = (0b0000_1001),
    /// Response of a ping frame.
    Pong = (0b0000_1010),
  }
}

impl OpCode {
  /// If this instance is of type [`OpCode::Close`].
  #[inline]
  pub const fn is_close(self) -> bool {
    matches!(self, Self::Close)
  }

  /// If this instance is of type [`OpCode::Binary`] or [`OpCode::Text`].
  #[inline]
  pub const fn is_data(self) -> bool {
    matches!(self, Self::Binary | Self::Text)
  }

  /// If this instance is of type [`OpCode::Close`], [`OpCode::Ping`] or [`OpCode::Pong`].
  #[inline]
  pub const fn is_control(self) -> bool {
    matches!(self, Self::Close | Self::Ping | Self::Pong)
  }

  /// If this instance is of type [`OpCode::Text`].
  #[inline]
  pub const fn is_text(self) -> bool {
    matches!(self, Self::Text)
  }
}
