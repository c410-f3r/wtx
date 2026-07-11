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

#[cfg(any(feature = "http2-client-pool", feature = "http2-server-framework"))]
pub(crate) fn push_h2_alpn<TM>(tc: &mut crate::tls::TlsConfig<TM>) -> crate::Result<()> {
  tc.alpn_mut().get_or_insert_default().protocol_name_list.push("h2".as_bytes().try_into()?)?;
  Ok(())
}
