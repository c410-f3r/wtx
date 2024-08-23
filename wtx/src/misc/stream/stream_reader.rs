use core::future::Future;

/// A stream of values sent asynchronously.
pub trait StreamReader {
  /// Pulls some bytes from this source into the specified buffer, returning how many bytes
  /// were read.
  fn read(&mut self, bytes: &mut [u8]) -> impl Future<Output = crate::Result<usize>>;

  /// Reads and at the same time discards exactly `len` bytes.
  #[inline]
  fn read_skip(&mut self, len: usize) -> impl Future<Output = crate::Result<()>> {
    async move {
      let mut buffer = [0; 32];
      let mut counter = len;
      for _ in 0..len {
        if counter == 0 {
          break;
        }
        let slice = if let Some(el) = buffer.get_mut(..counter) { el } else { &mut buffer[..] };
        let read = self.read(slice).await?;
        if read == 0 {
          return Err(crate::Error::UnexpectedStreamReadEOF);
        }
        counter = counter.wrapping_sub(read);
      }
      Ok(())
    }
  }
}

impl<T> StreamReader for &mut T
where
  T: StreamReader,
{
  #[inline]
  async fn read(&mut self, bytes: &mut [u8]) -> crate::Result<usize> {
    (**self).read(bytes).await
  }
}

impl StreamReader for () {
  #[inline]
  async fn read(&mut self, _: &mut [u8]) -> crate::Result<usize> {
    Ok(0)
  }
}
