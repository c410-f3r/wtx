use core::future::Future;

/// A stream of values written asynchronously.
pub trait StreamWriter {
  /// Attempts to write ***all*** `bytes`.
  fn write_all(&mut self, bytes: &[u8]) -> impl Future<Output = crate::Result<()>>;

  /// Attempts to write ***all*** `bytes` of all slices in a single syscall.
  ///
  /// # Panics
  ///
  /// If the length of the outermost slice is greater than 8.
  fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> impl Future<Output = crate::Result<()>>;
}

impl<T> StreamWriter for &mut T
where
  T: StreamWriter,
{
  #[inline]
  async fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
    (**self).write_all(bytes).await
  }

  #[inline]
  async fn write_all_vectored(&mut self, bytes: &[&[u8]]) -> crate::Result<()> {
    (**self).write_all_vectored(bytes).await
  }
}

impl StreamWriter for () {
  #[inline]
  async fn write_all(&mut self, _: &[u8]) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn write_all_vectored(&mut self, _: &[&[u8]]) -> crate::Result<()> {
    Ok(())
  }
}
