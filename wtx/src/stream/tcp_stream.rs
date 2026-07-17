use crate::{misc::TcpParams, stream::Stream};
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Reliable, ordered, and error-checked delivery of a stream of bytes.
pub trait TcpStream: Sized + Stream {
  /// The executor associated with this stream.
  type Executor;

  /// Establishes a new TCP connection to the specified address.
  fn connect(addr: (&str, u16), tcp_params: TcpParams)
  -> impl Future<Output = crate::Result<Self>>;

  /// Returns the socket address of the remote peer.
  fn peer_addr(&self) -> crate::Result<SocketAddr>;
}

impl TcpStream for () {
  type Executor = ();

  #[inline]
  async fn connect(_: (&str, u16), _: TcpParams) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn peer_addr(&self) -> crate::Result<SocketAddr> {
    Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0)))
  }
}
