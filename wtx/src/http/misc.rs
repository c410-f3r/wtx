use crate::{
  collection::Vector,
  http::{Headers, KnownHeaderName, Method, Protocol, ReqResBuffer, Request},
};

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

/// Used as an auxiliary tool for tests.
#[inline]
pub fn state_tuple_wo_uri<CA, SA>(
  (conn_aux, stream_aux, req): &mut (CA, SA, Request<ReqResBuffer>),
) -> (&mut CA, &mut SA, Request<(&mut Vector<u8>, &mut Headers)>) {
  let (body, headers, _) = req.rrd.parts_mut();
  (conn_aux, stream_aux, Request { method: req.method, rrd: (body, headers), version: req.version })
}
