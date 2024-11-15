use crate::{
  http::{Headers, KnownHeaderName, Method, Protocol, ReqResBuffer, Request},
  misc::Vector,
};

/// Verifies if the initial received headers represent a WebSocket connection.
#[inline]
pub fn is_web_socket_handshake(
  headers: &Headers,
  method: Method,
  protocol: Option<Protocol>,
) -> bool {
  let bytes = KnownHeaderName::SecWebsocketVersion.into();
  method == Method::Connect
    && protocol == Some(Protocol::WebSocket)
    && headers.get_by_name(bytes).map(|el| el.value) == Some(b"13")
}

/// Used as an auxiliary tool for tests.
#[inline]
pub fn state_tuple_wo_uri<CA, SA>(
  (conn_aux, stream_aux, req): &mut (CA, SA, Request<ReqResBuffer>),
) -> (&mut CA, &mut SA, Request<(&mut Vector<u8>, &mut Headers)>) {
  let (body, headers, _) = req.rrd.parts_mut();
  (conn_aux, stream_aux, Request { method: req.method, rrd: (body, headers), version: req.version })
}
