use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Connectionless unreliable delivery of a stream of bytes.
pub trait UdpStream: Sized {
  /// Receives a single datagram message on the socket. On success, returns the number
  /// of bytes read and the origin.
  fn recv_from(
    &mut self,
    buffer: &mut [u8],
  ) -> impl Future<Output = crate::Result<(usize, SocketAddr)>>;

  /// Sends data on the socket to the given address. On success, returns the
  /// number of bytes written.
  fn send_to(
    &mut self,
    bytes: &mut [u8],
    addr: SocketAddr,
  ) -> impl Future<Output = crate::Result<usize>>;
}

impl<T> UdpStream for &mut T
where
  T: UdpStream,
{
  #[inline]
  async fn recv_from(&mut self, buffer: &mut [u8]) -> crate::Result<(usize, SocketAddr)> {
    (**self).recv_from(buffer).await
  }

  #[inline]
  async fn send_to(&mut self, bytes: &mut [u8], addr: SocketAddr) -> crate::Result<usize> {
    (**self).send_to(bytes, addr).await
  }
}

impl UdpStream for () {
  #[inline]
  async fn send_to(&mut self, _: &mut [u8], _: SocketAddr) -> crate::Result<usize> {
    Ok(0)
  }

  #[inline]
  async fn recv_from(&mut self, _: &mut [u8]) -> crate::Result<(usize, SocketAddr)> {
    Ok((0, SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_bits(0), 0))))
  }
}
