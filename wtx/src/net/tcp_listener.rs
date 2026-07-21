use crate::net::{TcpParams, TcpStream, ToSocketAddrs};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Reliable, ordered, and error-checked listening of a stream of bytes.
pub trait TcpListener: Sized {
  /// The TCP stream type produced by this listener.
  type TcpStream: TcpStream;

  /// Binds a new TCP listener to the specified address and port.
  fn bind<A>(addr: A, tcp_params: TcpParams) -> impl Future<Output = crate::Result<Self>>
  where
    A: ToSocketAddrs;

  /// Accepts a new incoming TCP connection.
  fn accept(
    &self,
    tcp_params: TcpParams,
  ) -> impl Future<Output = crate::Result<(Self::TcpStream, SocketAddr)>>;
}

impl TcpListener for () {
  type TcpStream = ();

  #[inline]
  async fn bind<A>(_: A, _: TcpParams) -> crate::Result<Self>
  where
    A: ToSocketAddrs,
  {
    Ok(())
  }

  #[inline]
  async fn accept(&self, _: TcpParams) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok(((), SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0))))
  }
}
