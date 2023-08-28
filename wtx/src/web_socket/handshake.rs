//! Handshake

#[cfg(feature = "web-socket-hyper")]
pub(super) mod hyper;
mod misc;
pub(super) mod raw;
#[cfg(test)]
mod tests;

#[cfg(feature = "web-socket-hyper")]
pub use self::hyper::{UpgradeFutHyper, WebSocketHandshakeHyper, WebSocketUpgradeHyper};
use crate::web_socket::{Stream, WebSocketClient, WebSocketServer};
#[cfg(feature = "async-trait")]
use alloc::boxed::Box;
use core::future::Future;
#[cfg(feature = "web-socket-handshake")]
pub use raw::{WebSocketAcceptRaw, WebSocketHandshakeRaw};

/// Manages incoming data to establish WebSocket connections.
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait WebSocketAccept<RB> {
    /// Specific implementation response.
    type Response;
    /// Specific implementation stream.
    type Stream: Stream;

    /// Try to upgrade a received request to a WebSocket connection.
    async fn accept(self) -> crate::Result<(Self::Response, WebSocketServer<RB, Self::Stream>)>;
}

/// Initial negotiation sent by a client to start exchanging frames.
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait WebSocketHandshake<RB> {
    /// Specific implementation response.
    type Response;
    /// Specific implementation stream.
    type Stream: Stream;

    /// Performs the client handshake.
    async fn handshake(self) -> crate::Result<(Self::Response, WebSocketClient<RB, Self::Stream>)>;
}

/// Manages the upgrade of already established requests into WebSocket connections.
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait WebSocketUpgrade {
    /// Specific implementation response.
    type Response;
    /// Specific implementation stream.
    type Stream: Stream;
    /// Specific implementation future that resolves to [WebSocketServer].
    type Upgrade: Future<Output = crate::Result<Self::Stream>>;

    /// Try to upgrade a received request to a WebSocket connection.
    fn upgrade(self) -> crate::Result<(Self::Response, Self::Upgrade)>;
}

/// Necessary to decode incoming bytes of responses or requests.
#[derive(Debug)]
pub struct HeadersBuffer<'buffer, const N: usize> {
    pub(crate) headers: [httparse::Header<'buffer>; N],
}

impl<const N: usize> Default for HeadersBuffer<'_, N> {
    #[inline]
    fn default() -> Self {
        Self {
            headers: core::array::from_fn(|_| httparse::EMPTY_HEADER),
        }
    }
}
