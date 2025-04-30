use crate::stream::{StreamReader, StreamWriter};
use embassy_net::tcp::TcpSocket;

impl StreamReader for TcpSocket<'_> {
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok((*self).read(bytes).await?)
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
