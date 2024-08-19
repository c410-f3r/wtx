use crate::misc::{StreamReader, StreamWithTls, StreamWriter};
use ring::digest::{self, Digest};
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
    Ok(match conn.peer_certificates() {
      Some([cert, ..]) => {
        #[cfg(feature = "x509-certificate")]
        let algorithm = {
          use x509_certificate::{DigestAlgorithm, SignatureAlgorithm};
          let x509_cer = x509_certificate::X509Certificate::from_der(cert)?;
          let Some(sa) = x509_cer.signature_algorithm() else {
            return Ok(None);
          };
          match sa {
            SignatureAlgorithm::EcdsaSha256
            | SignatureAlgorithm::RsaSha1
            | SignatureAlgorithm::RsaSha256 => &digest::SHA256,
            SignatureAlgorithm::EcdsaSha384 | SignatureAlgorithm::RsaSha384 => &digest::SHA384,
            SignatureAlgorithm::Ed25519 => &digest::SHA512,
            SignatureAlgorithm::NoSignature(da) => match da {
              DigestAlgorithm::Sha1 | DigestAlgorithm::Sha256 => &digest::SHA256,
              DigestAlgorithm::Sha384 => &digest::SHA384,
              DigestAlgorithm::Sha512 => &digest::SHA512,
            },
            SignatureAlgorithm::RsaSha512 => &digest::SHA512,
          }
        };
        #[cfg(not(feature = "x509-certificate"))]
        let algorithm = &digest::SHA256;
        Some(digest::digest(algorithm, cert.as_ref()))
      }
      _ => None,
    })
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
    Ok(match conn.peer_certificates() {
      Some([cert, ..]) => Some(digest::digest(&digest::SHA256, cert)),
      _ => None,
    })
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
