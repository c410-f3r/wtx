//! Optioned high-level abstraction for servers. You can use one of listed suggestions or
//! create your own.
//
// FIXME(STABLE): Return type notation

#[cfg(all(feature = "http2", feature = "tokio"))]
mod http2_tokio;
#[cfg(all(feature = "tokio", feature = "web-socket-handshake"))]
mod web_socket_tokio;

/// Optioned abstractions of low-level servers.
#[derive(Debug)]
pub struct OptionedServer;
