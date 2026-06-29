use crate::{
  collections::MaybeUninitSlice,
  stream::{StreamCommon, StreamReadItem, StreamReader, StreamWriter},
};
use core::{future, num::NonZeroUsize};
use std::os::unix::prelude::{AsRawFd, RawFd};

/// Kernel TLS stream
///
/// <https://docs.kernel.org/networking/tls-offload.html>
#[derive(Debug)]
pub struct KtlsStream<IO> {
  io: IO,
}

impl<IO> StreamCommon for KtlsStream<IO> {}

impl<IO> StreamReader for KtlsStream<IO>
where
  IO: AsRawFd + StreamReader,
{
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
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
  fn write_all_vectored(&mut self, _: &[&[u8]]) -> impl Future<Output = crate::Result<()>> {
    future::ready(Ok(()))
  }
}

impl<IO> AsRawFd for KtlsStream<IO>
where
  IO: AsRawFd,
{
  #[inline]
  fn as_raw_fd(&self) -> RawFd {
    self.io.as_raw_fd()
  }
}
