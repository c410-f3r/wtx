//! Initial data negotiation on both client and server sides to start exchanging frames.

#[cfg(feature = "web-socket-handshake")]
mod misc;
mod raw;
#[cfg(test)]
mod tests;

use crate::{
  web_socket::{WebSocketClient, WebSocketServer},
  AsyncBounds,
};
use core::future::Future;
pub use raw::{WebSocketAcceptRaw, WebSocketConnectRaw};

/// Reads external data to figure out if incoming requests can be accepted as WebSocket connections.
pub trait WebSocketAccept<NC, PB, RNG, S> {
  /// Reads external data to figure out if incoming requests can be accepted as WebSocket connections.
  fn accept(
    self,
  ) -> impl AsyncBounds + Future<Output = crate::Result<WebSocketServer<NC, PB, RNG, S>>>;
}

/// Initial negotiation sent by a client to start a WebSocket connection.
pub trait WebSocketConnect<NC, PB, RNG, S> {
  /// Specific implementation response.
  type Response;

  /// Initial negotiation sent by a client to start a WebSocket connection.
  fn connect(
    self,
  ) -> impl AsyncBounds + Future<Output = crate::Result<(Self::Response, WebSocketClient<NC, PB, RNG, S>)>>;
}

/// Necessary to decode incoming bytes of responses or requests.
#[derive(Debug)]
pub struct HeadersBuffer<H, const N: usize> {
  #[allow(unused)]
  pub(crate) headers: [H; N],
}

#[cfg(feature = "httparse")]
impl<const N: usize> Default for HeadersBuffer<httparse::Header<'_>, N> {
  #[inline]
  fn default() -> Self {
    Self { headers: core::array::from_fn(|_| httparse::EMPTY_HEADER) }
  }
}
