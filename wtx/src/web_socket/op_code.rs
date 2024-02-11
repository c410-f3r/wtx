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
  #[inline]
  pub(crate) fn is_control(self) -> bool {
    matches!(self, OpCode::Close | OpCode::Ping | OpCode::Pong)
  }

  #[inline]
  pub(crate) fn is_text(self) -> bool {
    matches!(self, OpCode::Text)
  }
}
