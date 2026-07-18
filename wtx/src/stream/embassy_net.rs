use crate::{
  collections::MaybeUninitSlice,
  stream::{Stream, StreamCommon, StreamReader, StreamWriter, UdpStream},
};
use core::{net::SocketAddr, num::NonZeroUsize};
use embassy_net::{
  IpEndpoint,
  tcp::{TcpReader, TcpSocket, TcpWriter},
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
impl<'socket> Stream for &'socket mut TcpSocket<'_> {
  type BridgeOwned = ();
  type ReadHalfOwned = TcpReader<'socket>;
  type WriteHalfOwned = TcpWriter<'socket>;

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    let (reader, writer) = self.split();
    Ok(((), reader, writer))
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
    self.flush().await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    for elem in bytes {
      self.write_all(elem).await?;
    }
    self.flush().await?;
    Ok(())
  }
}

impl StreamCommon for TcpReader<'_> {}
impl StreamReader for TcpReader<'_> {
  #[inline]
  async fn read(
    &mut self,
    mut bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<Option<NonZeroUsize>> {
    Ok(NonZeroUsize::new((*self).read(bytes.initialize_all_bytes()).await?))
  }
}

impl StreamCommon for TcpWriter<'_> {}
impl StreamWriter for TcpWriter<'_> {
  #[inline]
  async fn write_all(&mut self, mut bytes: &[u8]) -> crate::Result<()> {
    _local_write_all!(bytes, Self::write(self, bytes).await);
    self.flush().await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    for elem in bytes {
      self.write_all(elem).await?;
    }
    self.flush().await?;
    Ok(())
  }
}

impl UdpStream for UdpSocket<'_> {
  #[inline]
  async fn recv_from(&mut self, buffer: &mut [u8]) -> crate::Result<(usize, SocketAddr)> {
    let (read, metadata) = (*self).recv_from(buffer).await?;
    Ok((read, metadata.endpoint.into()))
  }

  #[inline]
  async fn send_to(&mut self, bytes: &mut [u8], addr: SocketAddr) -> crate::Result<usize> {
    (*self).send_to(bytes, UdpMetadata::from(IpEndpoint::from(addr))).await?;
    self.flush().await;
    Ok(bytes.len())
  }
}
