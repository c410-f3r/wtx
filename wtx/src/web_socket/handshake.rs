//! Initial data negotiation on both client and server sides to start exchanging frames.

#[cfg(feature = "web-socket-handshake")]
mod misc;
mod raw;
#[cfg(all(feature = "tokio", feature = "web-socket-handshake", test))]
mod tests;

use crate::{
  http::GenericRequest,
  web_socket::{WebSocketClient, WebSocketServer},
};
use core::future::Future;
pub use raw::{WebSocketAcceptRaw, WebSocketConnectRaw};

/// Reads external data to figure out if incoming requests can be accepted as WebSocket connections.
pub trait WebSocketAccept<NC, RNG, S, WSC> {
  /// Reads external data to figure out if incoming requests can be accepted as WebSocket connections.
  fn accept(
    self,
    cb: impl FnOnce(&dyn GenericRequest) -> bool,
  ) -> impl Future<Output = crate::Result<WebSocketServer<NC, RNG, S, WSC>>>;
}

/// Initial negotiation sent by a client to start a WebSocket connection.
pub trait WebSocketConnect<NC, RNG, S, WSC> {
  /// Specific implementation response.
  type Response;

  /// Initial negotiation sent by a client to start a WebSocket connection.
  fn connect<'bytes>(
    self,
    headers: impl IntoIterator<Item = (&'bytes [u8], &'bytes [u8])>,
  ) -> impl Future<Output = crate::Result<(Self::Response, WebSocketClient<NC, RNG, S, WSC>)>>;
}

/// Necessary to decode incoming bytes of responses or requests.
#[derive(Debug)]
pub struct HeadersBuffer<H, const N: usize> {
  pub(crate) _headers: [H; N],
}

#[cfg(feature = "httparse")]
impl<const N: usize> Default for HeadersBuffer<httparse::Header<'_>, N> {
  #[inline]
  fn default() -> Self {
    Self { _headers: [httparse::EMPTY_HEADER; N] }
  }
}
