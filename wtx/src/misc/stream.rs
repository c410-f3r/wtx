macro_rules! _local_write_all {
  ($bytes:expr, $write:expr) => {{
    while !$bytes.is_empty() {
      match $write {
        Err(e) => return Err(e.into()),
        Ok(0) => return Err(crate::Error::UnexpectedEOF),
        Ok(n) => $bytes = $bytes.get(n..).unwrap_or_default(),
      }
    }
  }};
}

macro_rules! _local_write_all_vectored {
  ($bytes:expr, |$io_slices:ident| $write:expr) => {{
    let mut buffer = [std::io::IoSlice::new(&[]); N];
    let mut $io_slices = crate::misc::stream::convert_to_io_slices(&mut buffer, $bytes);
    while !$io_slices.is_empty() {
      match $write {
        Err(e) => return Err(e.into()),
        Ok(0) => return Err(crate::Error::UnexpectedEOF),
        Ok(n) => super::advance_slices(&mut &$bytes[..], &mut $io_slices, n),
      }
    }
  }};
}

use crate::misc::{AsyncBounds, Lease};
use alloc::vec::Vec;
use core::{cmp::Ordering, future::Future};

/// A stream of values produced asynchronously.
pub trait Stream {
  /// Pulls some bytes from this source into the specified buffer, returning how many bytes
  /// were read.
  fn read(&mut self, bytes: &mut [u8]) -> impl AsyncBounds + Future<Output = crate::Result<usize>>;

  /// Attempts to write ***all*** `bytes`.
  fn write_all(&mut self, bytes: &[u8]) -> impl AsyncBounds + Future<Output = crate::Result<()>>;

  /// Attempts to write ***all*** `bytes` of all slices in a single syscall.
  ///
  /// # Panics
  ///
  /// If the length of the outermost slice is greater than 8.
  fn write_all_vectored<const N: usize>(
    &mut self,
    bytes: [&[u8]; N],
  ) -> impl AsyncBounds + Future<Output = crate::Result<()>>;
}

/// Transport Layer Security
pub trait TlsStream: Stream {
  /// Channel binding data defined in [RFC 5929].
  ///
  /// [RFC 5929]: https://tools.ietf.org/html/rfc5929
  type TlsServerEndPoint: Lease<[u8]>;

  /// See `Self::TlsServerEndPoint`.
  fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>>;
}

impl Stream for () {
  #[inline]
  async fn read(&mut self, _: &mut [u8]) -> crate::Result<usize> {
    Ok(0)
  }

  #[inline]
  async fn write_all(&mut self, _: &[u8]) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn write_all_vectored<const N: usize>(&mut self, _: [&[u8]; N]) -> crate::Result<()> {
    Ok(())
  }
}

impl<T> Stream for &mut T
where
  T: AsyncBounds + Stream,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    (**self).read(bytes).await
  }

  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    (**self).write_all(bytes).await
  }

  #[inline]
  async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
    (**self).write_all_vectored(bytes).await
  }
}

/// Stores written data to transfer when read.
#[derive(Debug, Default)]
pub struct BytesStream {
  buffer: Vec<u8>,
  idx: usize,
}

impl BytesStream {
  /// Empties the internal buffer.
  #[inline]
  pub fn clear(&mut self) {
    self.buffer.clear();
    self.idx = 0;
  }
}

impl Stream for BytesStream {
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    let working_buffer = self.buffer.get(self.idx..).unwrap_or_default();
    let working_buffer_len = working_buffer.len();
    Ok(match working_buffer_len.cmp(&bytes.len()) {
      Ordering::Less => {
        bytes.get_mut(..working_buffer_len).unwrap_or_default().copy_from_slice(working_buffer);
        self.clear();
        working_buffer_len
      }
      Ordering::Equal => {
        bytes.copy_from_slice(working_buffer);
        self.clear();
        working_buffer_len
      }
      Ordering::Greater => {
        bytes.copy_from_slice(working_buffer.get(..bytes.len()).unwrap_or_default());
        self.idx = self.idx.wrapping_add(bytes.len());
        bytes.len()
      }
    })
  }

  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    self.buffer.extend_from_slice(bytes);
    Ok(())
  }

  #[inline]
  async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
    for elem in bytes {
      self.buffer.extend_from_slice(elem);
    }
    Ok(())
  }
}

#[cfg(feature = "async-std")]
mod async_std {
  use crate::misc::Stream;
  use async_std::{
    io::{ReadExt, WriteExt},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as ReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as WriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }

  #[cfg(unix)]
  impl Stream for async_std::os::unix::net::UnixStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as ReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as WriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }
}

#[cfg(all(feature = "embassy-net", not(feature = "async-send")))]
mod embassy_net {
  use crate::misc::Stream;
  use embassy_net::tcp::TcpSocket;

  impl Stream for TcpSocket<'_> {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok((*self).read(bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, mut bytes: &[u8]) -> crate::Result<()> {
      _local_write_all!(bytes, Self::write(self, bytes).await);
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      for elem in bytes {
        self.write_all(elem).await?;
      }
      Ok(())
    }
  }
}

#[cfg(all(feature = "embedded-tls", not(feature = "async-send")))]
mod embedded_tls {
  use crate::misc::{stream::TlsStream, Stream};
  use embedded_io_async::{Read, Write};
  use embedded_tls::{TlsCipherSuite, TlsConnection};

  impl<'any, S, C> Stream for TlsConnection<'any, S, C>
  where
    C: TlsCipherSuite + 'static,
    S: Read + Write + 'any,
  {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as Read>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as Write>::write_all(self, bytes).await?;
      self.flush().await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      for elem in bytes {
        <Self as Stream>::write_all(self, elem).await?;
      }
      Ok(())
    }
  }

  impl<'any, S, C> TlsStream for TlsConnection<'any, S, C>
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
}

#[cfg(all(feature = "glommio", not(feature = "async-send")))]
mod glommio {
  use crate::misc::Stream;
  use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
  use glommio::net::TcpStream;

  impl Stream for TcpStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      for elem in bytes {
        <Self as Stream>::write_all(self, elem).await?;
      }
      Ok(())
    }
  }

  #[cfg(unix)]
  impl Stream for glommio::net::UnixStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      for elem in bytes {
        <Self as Stream>::write_all(self, elem).await?;
      }
      Ok(())
    }
  }
}

#[cfg(feature = "smol")]
mod smol {
  use crate::misc::Stream;
  use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }

  #[cfg(unix)]
  impl Stream for smol::net::unix::UnixStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }
}

#[cfg(feature = "smoltcp")]
mod smoltcp {
  use crate::misc::Stream;
  use smoltcp::socket::tcp::Socket;

  impl Stream for Socket<'_> {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(self.recv_slice(bytes)?)
    }

    #[inline]
    async fn write_all(&mut self, mut bytes: &[u8]) -> crate::Result<()> {
      _local_write_all!(bytes, self.send_slice(bytes));
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      for elem in bytes {
        self.write_all(elem).await?;
      }
      Ok(())
    }
  }
}

#[cfg(feature = "std")]
mod _std {
  use crate::misc::Stream;
  use std::{
    io::{Read, Write},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as Read>::read(self, bytes)?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as Write>::write_all(self, bytes)?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices));
      Ok(())
    }
  }

  #[cfg(unix)]
  impl Stream for std::os::unix::net::UnixStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as Read>::read(self, bytes)?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as Write>::write_all(self, bytes)?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices));
      Ok(())
    }
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::misc::Stream;
  use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }

  #[cfg(unix)]
  impl Stream for tokio::net::UnixStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }
}

#[cfg(feature = "tokio-rustls")]
mod tokio_rustls {
  use crate::misc::{stream::TlsStream, AsyncBounds, Stream};
  use ring::digest::{self, Digest};
  use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

  impl<T> Stream for tokio_rustls::client::TlsStream<T>
  where
    T: AsyncBounds + AsyncRead + AsyncWrite + Unpin,
  {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }

  impl<T> TlsStream for tokio_rustls::client::TlsStream<T>
  where
    T: AsyncBounds + AsyncRead + AsyncWrite + Unpin,
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

  impl<T> Stream for tokio_rustls::server::TlsStream<T>
  where
    T: AsyncBounds + AsyncRead + AsyncWrite + Unpin,
  {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
      Ok(<Self as AsyncReadExt>::read(self, bytes).await?)
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      <Self as AsyncWriteExt>::write_all(self, bytes).await?;
      Ok(())
    }

    #[inline]
    async fn write_all_vectored<const N: usize>(&mut self, bytes: [&[u8]; N]) -> crate::Result<()> {
      _local_write_all_vectored!(bytes, |io_slices| self.write_vectored(io_slices).await);
      Ok(())
    }
  }

  impl<T> TlsStream for tokio_rustls::server::TlsStream<T>
  where
    T: AsyncBounds + AsyncRead + AsyncWrite + Unpin,
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
}

#[allow(
  // False-positive
  clippy::mut_mut
)]
#[cfg(feature = "std")]
#[inline]
fn advance_slices<'bytes>(
  bytes: &mut &[&'bytes [u8]],
  io_slices: &mut &mut [std::io::IoSlice<'bytes>],
  written: usize,
) {
  let mut first_slice_idx = written;
  let mut slices_idx: usize = 0;
  for io_slice in io_slices.iter() {
    let Some(diff) = first_slice_idx.checked_sub(io_slice.len()) else {
      break;
    };
    first_slice_idx = diff;
    slices_idx = slices_idx.wrapping_add(1);
  }
  let Some((local_bytes @ [first_bytes, ..], local_io_slices)) = bytes
    .get(slices_idx..)
    .and_then(|el| Some((el, core::mem::take(io_slices).get_mut(slices_idx..)?)))
  else {
    return;
  };
  *bytes = local_bytes;
  *io_slices = local_io_slices;
  let [first_io_slices, ..] = io_slices else {
    return;
  };
  *first_io_slices = std::io::IoSlice::new(first_bytes.get(first_slice_idx..).unwrap_or_default());
}

#[cfg(feature = "std")]
#[inline]
fn convert_to_io_slices<'buffer, 'bytes, const N: usize>(
  buffer: &'buffer mut [std::io::IoSlice<'bytes>; N],
  elems: [&'bytes [u8]; N],
) -> &'buffer mut [std::io::IoSlice<'bytes>] {
  use std::io::IoSlice;
  const {
    if N > 8 {
      panic!("It is not possible to vectored write more than 8 slices");
    }
  }
  match elems.as_slice() {
    [a] => {
      buffer[0] = IoSlice::new(a);
      &mut buffer[..1]
    }
    [a, b] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      &mut buffer[..2]
    }
    [a, b, c] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      &mut buffer[..3]
    }
    [a, b, c, d] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      &mut buffer[..4]
    }
    [a, b, c, d, e] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      &mut buffer[..5]
    }
    [a, b, c, d, e, f] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      buffer[5] = IoSlice::new(f);
      &mut buffer[..6]
    }
    [a, b, c, d, e, f, g] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      buffer[5] = IoSlice::new(f);
      buffer[6] = IoSlice::new(g);
      &mut buffer[..7]
    }
    [a, b, c, d, e, f, g, h] => {
      buffer[0] = IoSlice::new(a);
      buffer[1] = IoSlice::new(b);
      buffer[2] = IoSlice::new(c);
      buffer[3] = IoSlice::new(d);
      buffer[4] = IoSlice::new(e);
      buffer[5] = IoSlice::new(f);
      buffer[6] = IoSlice::new(g);
      buffer[7] = IoSlice::new(h);
      &mut buffer[..8]
    }
    _ => &mut [],
  }
}
