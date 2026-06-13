use crate::stream::{StreamCommon, StreamReader, StreamWriter};
use std::os::unix::prelude::{AsRawFd, RawFd};

/// Kernel TLS stream
///
/// <https://docs.kernel.org/networking/tls-offload.html>
pub struct KtlsStream<IO> {
  io: IO,
}

impl<IO> StreamCommon for KtlsStream<IO> {
  const IS_KTLS: bool = true;
}

impl<IO> StreamReader for KtlsStream<IO>
where
  IO: AsRawFd + StreamReader,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    self.io.read(bytes).await
  }
}

impl<IO> StreamWriter for KtlsStream<IO>
where
  IO: AsRawFd + StreamWriter,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    self.io.write_all(bytes).await
  }

  #[inline]
  async fn write_all_vectored(&mut self, _: &[&[u8]]) -> crate::Result<()> {
    Ok(())
  }
}

impl<IO> AsRawFd for KtlsStream<IO>
where
  IO: AsRawFd,
{
  fn as_raw_fd(&self) -> RawFd {
    self.io.as_raw_fd()
  }
}
