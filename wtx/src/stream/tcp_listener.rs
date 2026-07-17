use crate::{misc::TcpParams, stream::TcpStream};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Reliable, ordered, and error-checked listening of a stream of bytes.
pub trait TcpListener: Sized {
  /// The TCP stream type produced by this listener.
  type TcpStream: TcpStream;

  /// Binds a new TCP listener to the specified address and port.
  fn bind(addr: (&str, u16), tcp_params: TcpParams) -> impl Future<Output = crate::Result<Self>>;

  /// Accepts a new incoming TCP connection.
  fn accept(
    &self,
    tcp_params: TcpParams,
  ) -> impl Future<Output = crate::Result<(Self::TcpStream, SocketAddr)>>;
}

impl TcpListener for () {
  type TcpStream = ();

  #[inline]
  async fn bind(_: (&str, u16), _: TcpParams) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  async fn accept(&self, _: TcpParams) -> crate::Result<(Self::TcpStream, SocketAddr)> {
    Ok(((), SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0))))
  }
}
