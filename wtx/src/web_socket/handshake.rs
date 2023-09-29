//! Initial data negotiation on both client and server sides to start exchanging frames.

#[cfg(feature = "web-socket-handshake")]
mod misc;
mod raw;
#[cfg(test)]
mod tests;

use crate::web_socket::{Stream, WebSocketClient, WebSocketServer};
use core::future::Future;
pub use raw::{WebSocketAcceptRaw, WebSocketConnectRaw};

/// Reads external data to figure out if incoming requests can be accepted as WebSocket connections.
pub trait WebSocketAccept<NC, PB, RNG> {
  /// Specific implementation stream.
  type Stream: Stream;

  /// Reads external data to figure out if incoming requests can be accepted as WebSocket connections.
  async fn accept(self) -> crate::Result<WebSocketServer<NC, PB, RNG, Self::Stream>>;
}

/// Initial negotiation sent by a client to start a WebSocket connection.
pub trait WebSocketConnect<NC, PB, RNG> {
  /// Specific implementation response.
  type Response;
  /// Specific implementation stream.
  type Stream: Stream;

  /// Initial negotiation sent by a client to start a WebSocket connection.
  async fn connect(
    self,
  ) -> crate::Result<(Self::Response, WebSocketClient<NC, PB, RNG, Self::Stream>)>;
}

/// Manages the upgrade of already established requests into WebSocket connections.
pub trait WebSocketUpgrade {
  /// Specific implementation response.
  type Response;
  /// Specific implementation stream.
  type Stream: Stream;
  /// Specific implementation future that resolves to [WebSocketServer].
  type Upgrade: Future<Output = crate::Result<Self::Stream>>;

  /// Manages the upgrade of already established requests into WebSocket connections.
  fn upgrade(self) -> crate::Result<(Self::Response, Self::Upgrade)>;
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
