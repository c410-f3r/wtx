use crate::stream::{StreamReader, StreamWriter};
use std::{
  io::{Read, Write},
  net::TcpStream,
};

impl StreamReader for TcpStream {
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(<Self as Read>::read(self, bytes)?)
  }
}

#[cfg(unix)]
impl StreamReader for std::os::unix::net::UnixStream {
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(<Self as Read>::read(self, bytes)?)
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
