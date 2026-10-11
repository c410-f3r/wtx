use crate::http::HttpError;

create_enum! {
  /// Specifies a tunneling protocol
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum Protocol<u8> {
    /// WebSocket
    WebSocket = (0, "WEBSOCKET" | "websocket")
  }
}

impl Protocol {
  /// A specialized `TryFrom` for the HTTP/2 wire protocol
  #[inline]
  pub fn from_canonical_bytes(bytes: &[u8]) -> crate::Result<Self> {
    Ok(match bytes {
      b"WEBSOCKET" => Self::WebSocket,
      _ => return Err(HttpError::UnexpectedProtocol.into()),
    })
  }
}
