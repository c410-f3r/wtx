//! Optioned high-level abstraction for servers. You can use one of listed suggestions or
//! create your own.
//
// FIXME(STABLE): Return type notation

#[cfg(all(feature = "http2", feature = "tokio"))]
mod http2_tokio;
#[cfg(all(feature = "tokio", feature = "web-socket-handshake"))]
mod web_socket_tokio;

/// Optioned abstractions of low-level servers.
#[derive(Clone, Copy, Debug)]
pub struct OptionedServer;

#[cfg(feature = "tokio")]
fn default_listener(addr: &str) -> crate::Result<tokio::net::TcpListener> {
  let address: core::net::SocketAddr = addr.parse()?;
  let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;
  socket.set_nonblocking(true)?;
  socket.set_reuse_address(true)?;
  #[cfg(unix)]
  socket.set_reuse_port(true)?;
  socket.set_tcp_nodelay(true)?;
  socket.bind(&address.into())?;
  socket.listen(4096)?;
  Ok(tokio::net::TcpListener::from_std(std::net::TcpListener::from(socket))?)
}
