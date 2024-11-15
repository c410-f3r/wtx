use crate::misc::{StreamReader, StreamWithTls, StreamWriter};
use embedded_io_async::{Read, Write};
use embedded_tls::{TlsCipherSuite, TlsConnection};

impl<'any, S, C> StreamReader for TlsConnection<'any, S, C>
where
  C: TlsCipherSuite + 'static,
  S: Read + Write + 'any,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(<Self as Read>::read(self, bytes).await?)
  }
}

impl<'any, S, C> StreamWriter for TlsConnection<'any, S, C>
where
  C: TlsCipherSuite + 'static,
  S: Read + Write + 'any,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    <Self as Write>::write_all(self, bytes).await?;
    self.flush().await?;
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    for elem in bytes {
      <Self as StreamWriter>::write_all(self, elem).await?;
    }
    Ok(())
  }
}

impl<'any, S, C> StreamWithTls for TlsConnection<'any, S, C>
where
  C: TlsCipherSuite + 'static,
  S: Read + Write + 'any,
{
  type TlsServerEndPoint = [u8; 0];

  #[inline]
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>> {
    Ok(None)
  }
}
