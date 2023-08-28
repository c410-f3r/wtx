macro_rules! create_enum {
  ($(#[$meta:meta])* $vis:vis enum $name:ident {
    $($(#[$variant_meta:meta])* $variant_ident:ident = $variant_value:expr,)*
  }) => {
    $(#[$meta])*
    $vis enum $name {
      $($(#[$variant_meta])* $variant_ident = $variant_value,)*
    }

    impl From<$name> for u8 {
      #[inline]
      fn from(from: $name) -> Self {
        match from {
          $($name::$variant_ident => $variant_value,)*
        }
      }
    }

    impl TryFrom<u8> for $name {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: u8) -> Result<Self, Self::Error> {
        match from {
          $(x if x == u8::from($name::$variant_ident) => Ok($name::$variant_ident),)*
          _ => Err(crate::web_socket::WebSocketError::InvalidOpCodeByte { provided: from }.into()),
        }
      }
    }
  }
}

create_enum! {
  /// Defines how to interpret the payload data.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  #[repr(u8)]
  pub enum OpCode {
    /// Continuation of a previous frame.
    Continuation = 0b0000_0000,
    /// UTF-8 text.
    Text = 0b0000_0001,
    /// Opaque bytes.
    Binary = 0b0000_0010,
    /// Connection is closed.
    Close = 0b0000_1000,
    /// Test reachability.
    Ping = 0b0000_1001,
    /// Response of a ping frame.
    Pong = 0b0000_1010,
  }
}

impl OpCode {
    #[inline]
    pub(crate) fn is_continuation(self) -> bool {
        matches!(self, OpCode::Continuation)
    }

    #[inline]
    pub(crate) fn is_control(self) -> bool {
        matches!(self, OpCode::Close | OpCode::Ping | OpCode::Pong)
    }

    #[inline]
    pub(crate) fn is_text(self) -> bool {
        matches!(self, OpCode::Text)
    }
}
