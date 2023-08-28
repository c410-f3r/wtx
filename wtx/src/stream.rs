use crate::misc::AsyncBounds;
#[cfg(feature = "async-trait")]
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// A stream of values produced asynchronously.
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait Stream {
    /// Pulls some bytes from this source into the specified buffer, returning how many bytes
    /// were read.
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize>;

    /// Attempts to write all elements of `bytes`.
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()>;
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<T> Stream for &mut T
where
    T: AsyncBounds + Stream,
{
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
        (*self).read(bytes).await
    }

    #[inline]
    async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
        (*self).write_all(bytes).await
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

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Stream for BytesStream {
    #[inline]
    async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
        let working_buffer = self.buffer.get(self.idx..).unwrap_or_default();
        let working_buffer_len = working_buffer.len();
        Ok(match working_buffer_len.cmp(&bytes.len()) {
            Ordering::Less => {
                bytes
                    .get_mut(..working_buffer_len)
                    .unwrap_or_default()
                    .copy_from_slice(working_buffer);
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

/// Does nothing.
#[derive(Debug)]
pub struct DummyStream;

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Stream for DummyStream {
    #[inline]
    async fn read(&mut self, _: &mut [u8]) -> crate::Result<usize> {
        Ok(0)
    }

    #[inline]
    async fn write_all(&mut self, _: &[u8]) -> crate::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "async-std")]
mod async_std {
    use crate::Stream;
    #[cfg(feature = "async-trait")]
    use alloc::boxed::Box;
    use async_std::{
        io::{ReadExt, WriteExt},
        net::TcpStream,
    };

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
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
}

#[cfg(all(feature = "glommio", not(feature = "async-trait")))]
mod glommio {
    use crate::Stream;
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
}

#[cfg(feature = "hyper")]
mod hyper {
    use crate::Stream;
    #[cfg(feature = "async-trait")]
    use alloc::boxed::Box;
    use hyper::upgrade::Upgraded;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
    impl Stream for Upgraded {
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
    use crate::Stream;
    #[cfg(feature = "async-trait")]
    use alloc::boxed::Box;
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
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
}

#[cfg(feature = "tokio")]
mod tokio {
    use crate::Stream;
    #[cfg(feature = "async-trait")]
    use alloc::boxed::Box;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
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
}

#[cfg(feature = "tokio-rustls")]
mod tokio_rustls {
    use crate::{misc::AsyncBounds, Stream};
    #[cfg(feature = "async-trait")]
    use alloc::boxed::Box;
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
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

    #[cfg_attr(feature = "async-trait", async_trait::async_trait)]
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
}
