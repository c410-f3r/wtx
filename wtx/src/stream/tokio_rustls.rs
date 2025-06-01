use crate::stream::{StreamReader, StreamWithTls, StreamWriter};
use ring::digest::{self, Digest};
use rustls_pki_types::CertificateDer;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

impl<T> StreamReader for tokio_rustls::client::TlsStream<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
  }
}

impl<T> StreamWithTls for tokio_rustls::client::TlsStream<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  type TlsServerEndPoint = Digest;

  #[inline]
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>> {
    let (_, conn) = self.get_ref();
    tls_server_end_point(conn.peer_certificates())
  }
}

impl<T> StreamWriter for tokio_rustls::client::TlsStream<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
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

impl<T> StreamReader for tokio_rustls::server::TlsStream<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
  }
}

impl<T> StreamWithTls for tokio_rustls::server::TlsStream<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  type TlsServerEndPoint = Digest;

  #[inline]
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>> {
    let (_, conn) = self.get_ref();
    tls_server_end_point(conn.peer_certificates())
  }
}

impl<T> StreamWriter for tokio_rustls::server::TlsStream<T>
where
  T: AsyncRead + AsyncWrite + Unpin,
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

fn tls_server_end_point(
  certs: Option<&[CertificateDer<'static>]>,
) -> crate::Result<Option<Digest>> {
  Ok(match certs {
    Some([cert, ..]) => Some(digest::digest(&digest::SHA256, cert.as_ref())),
    _ => None,
  })
}
