use crate::{
  collections::MaybeUninitSlice,
  stream::{Stream, StreamCommon, StreamReader, StreamWriter, UdpStream},
};
use core::{net::SocketAddr, num::NonZeroUsize};
use embassy_net::{
  IpEndpoint,
  tcp::TcpSocket,
  udp::{UdpMetadata, UdpSocket},
};

impl Stream for TcpSocket<'_> {
  type BridgeOwned = ();
  type ReadHalfOwned = ();
  type WriteHalfOwned = ();

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    Ok(((), (), ()))
  }
}

impl StreamCommon for TcpSocket<'_> {}

impl StreamReader for TcpSocket<'_> {
  #[inline]
  async fn read(
    &mut self,
    mut bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<Option<NonZeroUsize>> {
    Ok(NonZeroUsize::new((*self).read(bytes.initialize_all_bytes()).await?))
  }
}

impl StreamWriter for TcpSocket<'_> {
  #[inline]
  async fn write_all(&mut self, mut bytes: &[u8]) -> crate::Result<()> {
    _local_write_all!(bytes, Self::write(self, bytes).await);
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    for elem in bytes {
      self.write_all(elem).await?;
    }
    Ok(())
  }
}

impl UdpStream for UdpSocket<'_> {
  #[inline]
  async fn recv_from(&self, buffer: &mut [u8]) -> crate::Result<(usize, SocketAddr)> {
    let (read, metadata) = (*self).recv_from(buffer).await?;
    Ok((read, metadata.endpoint.into()))
  }

  #[inline]
  async fn send_to(&self, bytes: &mut [u8], addr: SocketAddr) -> crate::Result<usize> {
    (*self).send_to(bytes, UdpMetadata::from(IpEndpoint::from(addr))).await?;
    Ok(bytes.len())
  }
}
