use crate::misc::{StreamReader, StreamWriter};
use monoio_compat::{AsyncReadExt, AsyncWriteExt, TcpStreamCompat};

impl StreamReader for TcpStreamCompat {
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
  }
}

impl StreamWriter for TcpStreamCompat {
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
