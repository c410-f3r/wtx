//! Optioned high-level abstraction for servers. You can use one of listed suggestions or
//! create your own.

#[cfg(all(feature = "http2", feature = "tokio"))]
mod tokio_http2;
#[cfg(all(feature = "pool", feature = "tokio", feature = "web-socket-handshake"))]
mod tokio_web_socket;

/// Optioned high-level abstractions for low-level servers.
#[derive(Debug)]
pub struct LowLevelServer;
