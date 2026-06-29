use crate::{
  collections::MaybeUninitSlice,
  stream::{Stream, StreamCommon, StreamReadItem, StreamReader, StreamWriter},
};
use core::{
  num::NonZeroUsize,
  pin::Pin,
  task::{Context, Poll, ready},
};
use tokio::{
  io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf, ReadHalf, WriteHalf},
  net::{
    TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
  },
};

impl Stream for TcpStream {
  type BridgeOwned = ();
  type ReadHalfOwned = OwnedReadHalf;
  type WriteHalfOwned = OwnedWriteHalf;

  #[inline]
  fn into_split(
    self,
  ) -> crate::Result<(Self::BridgeOwned, Self::ReadHalfOwned, Self::WriteHalfOwned)> {
    let (read, write) = self.into_split();
    Ok(((), read, write))
  }
}

impl StreamCommon for OwnedReadHalf {}

impl StreamCommon for OwnedWriteHalf {}

impl<T> StreamCommon for ReadHalf<T> where T: AsyncRead {}

impl StreamCommon for TcpStream {}

#[cfg(unix)]
impl StreamCommon for tokio::net::UnixStream {}

impl<T> StreamCommon for WriteHalf<T> where T: AsyncWrite {}

impl StreamReader for OwnedReadHalf {
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    ReadFut::new(bytes.into_tokio_read_buf(), self).await
  }
}

impl<T> StreamReader for ReadHalf<T>
where
  T: AsyncRead,
{
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    ReadFut::new(bytes.into_tokio_read_buf(), self).await
  }
}

impl StreamReader for TcpStream {
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    ReadFut::new(bytes.into_tokio_read_buf(), self).await
  }
}

#[cfg(unix)]
impl StreamReader for tokio::net::UnixStream {
  #[inline]
  async fn read(
    &mut self,
    bytes: MaybeUninitSlice<'_, u8>,
  ) -> crate::Result<StreamReadItem<NonZeroUsize>> {
    ReadFut::new(bytes.into_tokio_read_buf(), self).await
  }
}

impl StreamWriter for OwnedWriteHalf {
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as AsyncWriteExt>::write_all(self, bytes).await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    _local_write_all_vectored!(bytes, self, |io_slices| self.write_vectored(io_slices).await);
    Ok(())
  }
}

impl StreamWriter for TcpStream {
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as AsyncWriteExt>::write_all(self, bytes).await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    _local_write_all_vectored!(bytes, self, |io_slices| self.write_vectored(io_slices).await);
    Ok(())
  }
}

#[cfg(unix)]
impl StreamWriter for tokio::net::UnixStream {
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as AsyncWriteExt>::write_all(self, bytes).await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    _local_write_all_vectored!(bytes, self, |io_slices| self.write_vectored(io_slices).await);
    Ok(())
  }
}

impl<T> StreamWriter for WriteHalf<T>
where
  T: AsyncWrite,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as AsyncWriteExt>::write_all(self, bytes).await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    _local_write_all_vectored!(bytes, self, |io_slices| self.write_vectored(io_slices).await);
    Ok(())
  }
}

struct ReadFut<'any, R: ?Sized> {
  read_buf: ReadBuf<'any>,
  reader: &'any mut R,
}

impl<'any, R: ?Sized> ReadFut<'any, R> {
  #[inline]
  const fn new(read_buf: ReadBuf<'any>, reader: &'any mut R) -> Self {
    Self { read_buf, reader }
  }
}

impl<R> Future for ReadFut<'_, R>
where
  R: AsyncRead + Unpin + ?Sized,
{
  type Output = crate::Result<StreamReadItem<NonZeroUsize>>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let Self { read_buf, reader } = &mut *self;
    ready!(Pin::new(reader).poll_read(cx, read_buf))?;
    Poll::Ready(Ok(StreamReadItem::from_opt(NonZeroUsize::new(read_buf.filled().len()))))
  }
}
