/// Manages a set of atomic operations.
pub trait TransactionManager: Sized {
  /// Executor
  type Executor;

  /// Starts the atomic save-point.
  fn begin(&mut self) -> impl Future<Output = crate::Result<()>>;

  /// Flushes all previously sent commands.
  fn commit(self) -> impl Future<Output = crate::Result<()>>;

  /// Returns the underlying executor.
  fn executor(&mut self) -> &mut Self::Executor;
}

impl TransactionManager for () {
  type Executor = ();

  #[inline]
  async fn begin(&mut self) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn commit(self) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  fn executor(&mut self) -> &mut Self::Executor {
    self
  }
}
