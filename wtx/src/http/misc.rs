use crate::http::{Headers, KnownHeaderName, Method, Protocol};

/// Verifies if the initial received HTTP/2 headers represent a WebSocket connection.
#[inline]
pub fn is_web_socket_handshake(
  headers: &Headers,
  method: Method,
  protocol: Option<Protocol>,
) -> bool {
  let header = KnownHeaderName::SecWebsocketVersion.into();
  method == Method::Connect
    && protocol == Some(Protocol::WebSocket)
    && headers.get_by_name(header).map(|el| el.value) == Some("13")
}
