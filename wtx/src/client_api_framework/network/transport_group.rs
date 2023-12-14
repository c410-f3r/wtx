use core::fmt::{Display, Formatter};

/// It is possible to have one or more transports that send data using the same protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportGroup {
  /// Transport group depending outside of `wtx`.
  Custom(&'static str),
  /// Hypertext Transfer Protocol
  HTTP,
  /// Mock or dummy implementations
  Stub,
  /// Transmission Control Protocol
  TCP,
  /// User Datagram Protocol
  UDP,
  /// WebSocket
  WebSocket,
}

impl From<TransportGroup> for &'static str {
  #[inline]
  fn from(from: TransportGroup) -> Self {
    match from {
      TransportGroup::Custom(elem) => elem,
      TransportGroup::HTTP => "HTTP",
      TransportGroup::Stub => "Stub",
      TransportGroup::TCP => "TCP",
      TransportGroup::UDP => "UDP",
      TransportGroup::WebSocket => "WebSocket",
    }
  }
}

impl Display for TransportGroup {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.write_str((*self).into())
  }
}
