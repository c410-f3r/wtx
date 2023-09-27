macro_rules! create_enum {
  ($(#[$meta:meta])* $vis:vis enum $name:ident {
    $($(#[$variant_meta:meta])* $variant_ident:ident = $variant_value:literal,)*
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
          _ => Err(crate::web_socket::WebSocketError::InvalidFromByte { provided: from }.into()),
        }
      }
    }
  }
}

macro_rules! debug {
  ($($tt:tt)+) => {
    #[cfg(feature = "tracing")]
    tracing::debug!($($tt)+);
  };
}
