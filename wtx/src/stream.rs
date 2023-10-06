use alloc::vec::Vec;
use core::{
  cmp::Ordering,
  future::{ready, Future, Ready},
};

/// A stream of values produced asynchronously.
pub trait Stream {
  /// Future of `read` method
  type Read<'read>: Future<Output = crate::Result<usize>> + 'read
  where
    Self: 'read;
  /// Future of `write` method
  type Write<'write>: Future<Output = crate::Result<()>> + 'write
  where
    Self: 'write;

  /// Pulls some bytes from this source into the specified buffer, returning how many bytes
  /// were read.
  fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut;

  /// Attempts to write all elements of `bytes`.
  fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut;
}

impl Stream for () {
  type Read<'read> = Ready<crate::Result<usize>>
  where
    Self: 'read;
  type Write<'write> = Ready<crate::Result<()>>
  where
    Self: 'write;

  #[inline]
  fn read<'bytes, 'fut, 'this>(&'this mut self, _: &'bytes mut [u8]) -> Self::Read<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut,
  {
    ready(Ok(0))
  }

  #[inline]
  fn write_all<'bytes, 'fut, 'this>(&'this mut self, _: &'bytes [u8]) -> Self::Write<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut,
  {
    ready(Ok(()))
  }
}

impl<T> Stream for &mut T
where
  T: Stream,
{
  type Read<'read> = T::Read<'read>
  where
    Self: 'read;
  type Write<'write> = T::Write<'write>
  where
    Self: 'write;

  #[inline]
  fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut,
  {
    (*self).read(bytes)
  }

  #[inline]
  fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut,
  {
    (*self).write_all(bytes)
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
  type Read<'read> = Ready<crate::Result<usize>>
  where
    Self: 'read;
  type Write<'write> = Ready<crate::Result<()>>
  where
    Self: 'write;

  #[inline]
  fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut,
  {
    ready({
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
    })
  }

  #[inline]
  fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
  where
    'bytes: 'fut,
    'this: 'fut,
    Self: 'fut,
  {
    ready({
      self.buffer.extend_from_slice(bytes);
      Ok(())
    })
  }
}

#[cfg(feature = "async-std")]
mod async_std {
  use crate::Stream;
  use async_std::{
    io::{ReadExt, WriteExt},
    net::TcpStream,
  };
  use core::future::Future;

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as ReadExt>::read(self, bytes).await?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as WriteExt>::write_all(self, bytes).await?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "glommio")]
mod glommio {
  use crate::Stream;
  use core::future::Future;
  use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
  use glommio::net::TcpStream;

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as AsyncReadExt>::read(self, bytes).await?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as AsyncWriteExt>::write_all(self, bytes).await?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "monoio")]
mod monoio {
  use crate::Stream;
  use core::future::Future;
  use monoio::{
    io::{AsyncReadRent, AsyncWriteRentExt},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        let (rslt, read) = AsyncReadRent::read(self, bytes.to_vec()).await;
        bytes.get_mut(..read.len()).unwrap_or_default().copy_from_slice(&read);
        Ok(rslt?)
      }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        let (rslt, _) = AsyncWriteRentExt::write_all(self, bytes.to_vec()).await;
        rslt?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "smol")]
mod smol {
  use crate::Stream;
  use core::future::Future;
  use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as AsyncReadExt>::read(self, bytes).await?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as AsyncWriteExt>::write_all(self, bytes).await?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "std")]
mod std {
  use crate::Stream;
  use core::future::Future;
  use std::{
    io::{Read, Write},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as Read>::read(self, bytes)?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as Write>::write_all(self, bytes)?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::Stream;
  use core::future::Future;
  use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
  };

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as AsyncReadExt>::read(self, bytes).await?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as AsyncWriteExt>::write_all(self, bytes).await?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "tokio-uring")]
mod tokio_uring {
  use crate::Stream;
  use core::future::Future;
  use tokio_uring::net::TcpStream;

  impl Stream for TcpStream {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        let (rslt, read) = TcpStream::read(self, bytes.to_vec()).await;
        bytes.get_mut(..read.len()).unwrap_or_default().copy_from_slice(&read);
        Ok(rslt?)
      }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        let (rslt, _) = TcpStream::write_all(self, bytes.to_vec()).await;
        rslt?;
        Ok(())
      }
    }
  }
}

#[cfg(feature = "tokio-rustls")]
mod tokio_rustls {
  use crate::Stream;
  use core::future::Future;
  use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

  impl<T> Stream for tokio_rustls::client::TlsStream<T>
  where
    T: AsyncRead + AsyncWrite + Unpin,
  {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as AsyncReadExt>::read(self, bytes).await?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as AsyncWriteExt>::write_all(self, bytes).await?;
        Ok(())
      }
    }
  }

  impl<T> Stream for tokio_rustls::server::TlsStream<T>
  where
    T: AsyncRead + AsyncWrite + Unpin,
  {
    type Read<'read> = impl Future<Output = crate::Result<usize>> + 'read
    where
      Self: 'read;
    type Write<'write> = impl Future<Output = crate::Result<()>> + 'write
    where
      Self: 'write;

    #[inline]
    fn read<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes mut [u8]) -> Self::Read<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async { Ok(<Self as AsyncReadExt>::read(self, bytes).await?) }
    }

    #[inline]
    fn write_all<'bytes, 'fut, 'this>(&'this mut self, bytes: &'bytes [u8]) -> Self::Write<'fut>
    where
      'bytes: 'fut,
      'this: 'fut,
      Self: 'fut,
    {
      async {
        <Self as AsyncWriteExt>::write_all(self, bytes).await?;
        Ok(())
      }
    }
  }
}
