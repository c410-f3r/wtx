use crate::net::{Stream, TcpParams, ToSocketAddrs};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Reliable, ordered, and error-checked delivery of a stream of bytes.
pub trait TcpStream: Sized + Stream {
  /// The executor associated with this stream.
  type Executor;

  /// Establishes a new TCP connection to the specified address.
  fn connect<A>(addr: A, tcp_params: TcpParams) -> impl Future<Output = crate::Result<Self>>
  where
    A: ToSocketAddrs;

  /// Returns the socket address of the remote peer.
  fn peer_addr(&self) -> crate::Result<SocketAddr>;
}

impl TcpStream for () {
  type Executor = ();

  #[inline]
  async fn connect<A>(_: A, _: TcpParams) -> crate::Result<Self>
  where
    A: ToSocketAddrs,
  {
    Ok(())
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0)))
  }
}
