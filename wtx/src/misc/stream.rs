use crate::misc::AsyncBounds;
use alloc::vec::Vec;
use core::{cmp::Ordering, future::Future};

/// A stream of values produced asynchronously.
pub trait Stream: AsyncBounds {
  /// Pulls some bytes from this source into the specified buffer, returning how many bytes
  /// were read.
  fn read(&mut self, bytes: &mut [u8]) -> impl AsyncBounds + Future<Output = crate::Result<usize>>;

  /// Attempts to write all elements of `bytes`.
  fn write_all(&mut self, bytes: &[u8]) -> impl AsyncBounds + Future<Output = crate::Result<()>>;
}

/// Transport Layer Security
pub trait TlsStream: Stream {
  /// Channel binding data defined in [RFC 5929].
  ///
  /// [RFC 5929]: https://tools.ietf.org/html/rfc5929
  type TlsServerEndPoint: AsRef<[u8]>;

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
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
      let mut buf = bytes;
      while !buf.is_empty() {
        match self.write(buf).await {
          Ok(0) => return Err(crate::Error::UnexpectedEOF),
          Ok(n) => buf = buf.get(n..).unwrap_or_default(),
          Err(e) => return Err(e.into()),
        }
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
  }
}

#[cfg(feature = "std")]
mod std {
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
  }
}

#[cfg(feature = "tokio-rustls")]
mod tokio_rustls {
  use crate::misc::{stream::TlsStream, AsyncBounds, Stream};
  use ring::digest;
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
  }

  impl<T> TlsStream for tokio_rustls::client::TlsStream<T>
  where
    T: AsyncBounds + AsyncRead + AsyncWrite + Unpin,
  {
    type TlsServerEndPoint = digest::Digest;

    #[inline]
    fn tls_server_end_point(&self) -> crate::Result<Option<Self::TlsServerEndPoint>> {
      let (_, conn) = self.get_ref();
      Ok(match conn.peer_certificates() {
        Some([cert, ..]) => Some(digest::digest(&digest::SHA256, cert)),
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
  }

  impl<T> TlsStream for tokio_rustls::server::TlsStream<T>
  where
    T: AsyncBounds + AsyncRead + AsyncWrite + Unpin,
  {
    type TlsServerEndPoint = digest::Digest;

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
