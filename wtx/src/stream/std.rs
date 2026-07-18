use crate::{
  collections::MaybeUninitSlice,
  stream::{Stream, StreamCommon, StreamReader, StreamWriter, UdpStream},
};
use core::{net::SocketAddr, num::NonZeroUsize};
use std::{
  io::{Read, Write},
  net::{TcpStream, UdpSocket},
};

impl Stream for TcpStream {
  type BridgeOwned = ();
  type ReadHalfOwned = TcpStream;
  type WriteHalfOwned = TcpStream;

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    Ok(((), self.try_clone()?, self))
  }
}

impl StreamCommon for TcpStream {}

#[cfg(unix)]
impl StreamCommon for std::os::unix::net::UnixStream {}

impl StreamReader for TcpStream {
  #[inline]
  async fn read(
    &mut self,
    mut bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<Option<NonZeroUsize>> {
    Ok(NonZeroUsize::new(<Self as Read>::read(self, bytes.initialize_all_bytes())?))
  }
}

#[cfg(unix)]
impl StreamReader for std::os::unix::net::UnixStream {
  #[inline]
  async fn read(
    &mut self,
    mut bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<Option<NonZeroUsize>> {
    Ok(NonZeroUsize::new(<Self as Read>::read(self, bytes.initialize_all_bytes())?))
  }
}

impl StreamWriter for TcpStream {
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as Write>::write_all(self, bytes)?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    _local_write_all_vectored!(bytes, self, |io_slices| self.write_vectored(io_slices));
    Ok(())
  }
}

#[cfg(unix)]
impl StreamWriter for std::os::unix::net::UnixStream {
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as Write>::write_all(self, bytes)?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    _local_write_all_vectored!(bytes, self, |io_slices| self.write_vectored(io_slices));
    Ok(())
  }
}

impl UdpStream for UdpSocket {
  #[inline]
  async fn recv_from(&mut self, buffer: &mut [u8]) -> crate::Result<(usize, SocketAddr)> {
    Ok((*self).recv_from(buffer)?)
  }

  #[inline]
  async fn send_to(&mut self, bytes: &mut [u8], addr: SocketAddr) -> crate::Result<usize> {
    Ok((*self).send_to(bytes, addr)?)
  }
}
