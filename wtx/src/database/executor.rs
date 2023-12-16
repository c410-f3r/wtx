//! Database

use crate::database::{Database, FromRecord, RecordValues, StmtId, TransactionManager};
use alloc::vec::Vec;
use core::future::Future;

/// A connection for executing database commands.
pub trait Executor {
  /// See [Database].
  type Database: Database;
  /// Manages atomic operations.
  type TransactionManager<'tm>: TransactionManager<Executor = Self>
  where
    Self: 'tm;

  /// Executes severals commands returning the number of affected records on each `cb` call.
  ///
  /// Commands are not cached or inspected for potential vulnerabilities.
  fn execute(&mut self, cmd: &str, cb: impl FnMut(u64)) -> impl Future<Output = crate::Result<()>>;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt_id` and then returns the number of affected records.
  fn execute_with_stmt<E, SI, RV>(
    &mut self,
    stmt_id: SI,
    rv: RV,
  ) -> impl Future<Output = Result<u64, E>>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt_id` and then returns a **single** record.
  fn fetch_with_stmt<E, SI, RV>(
    &mut self,
    stmt_id: SI,
    sv: RV,
  ) -> impl Future<Output = Result<<Self::Database as Database>::Record<'_>, E>>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt_id` and then returns a **set** of records.
  fn fetch_many_with_stmt<E, SI, RV>(
    &mut self,
    stmt_id: SI,
    sv: RV,
    cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> impl Future<Output = Result<<Self::Database as Database>::Records<'_>, E>>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId;

  /// Caches the passed command to create a statement, which speeds up subsequent calls that match
  /// the same `cmd`.
  ///
  /// The returned integer is an identifier of the added statement.
  fn prepare(&mut self, cmd: &str) -> impl Future<Output = crate::Result<u64>>;

  /// Retrieves a record and maps it to `T`. See [FromRecord].
  #[inline]
  fn simple_entity<E, SV, T>(&mut self, cmd: &str, sv: SV) -> impl Future<Output = Result<T, E>>
  where
    E: From<crate::Error>,
    T: for<'rec> FromRecord<E, <Self::Database as Database>::Record<'rec>>,
    SV: RecordValues<Self::Database, E>,
  {
    async move { T::from_record(self.fetch_with_stmt(cmd, sv).await?) }
  }

  /// Retrieves a set of records and maps them to the corresponding `T`. See [FromRecord].
  #[inline]
  fn simple_entities<E, SV, T>(
    &mut self,
    cmd: &str,
    results: &mut Vec<T>,
    sv: SV,
  ) -> impl Future<Output = Result<(), E>>
  where
    E: From<crate::Error>,
    SV: RecordValues<Self::Database, E>,
    T: for<'rec> FromRecord<E, <Self::Database as Database>::Record<'rec>>,
  {
    async move {
      let _records = self
        .fetch_many_with_stmt(cmd, sv, |record| {
          results.push(T::from_record(record)?);
          Ok::<_, E>(())
        })
        .await?;
      Ok(())
    }
  }

  /// Initially calls `begin` and the returns [Self::TransactionManager], which implies in an
  /// following obligatory `commit` call by the caller.
  fn transaction(&mut self) -> impl Future<Output = crate::Result<Self::TransactionManager<'_>>>;
}

impl Executor for () {
  type Database = ();
  type TransactionManager<'tm> = ();

  #[inline]
  async fn execute(&mut self, _: &str, _: impl FnMut(u64)) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn execute_with_stmt<E, SI, RV>(&mut self, _: SI, _: RV) -> Result<u64, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId,
  {
    Ok(0)
  }

  #[inline]
  async fn fetch_with_stmt<E, SI, RV>(
    &mut self,
    _: SI,
    _: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId,
  {
    Ok(())
  }

  #[inline]
  async fn fetch_many_with_stmt<E, SI, RV>(
    &mut self,
    _: SI,
    _: RV,
    _: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId,
  {
    Ok(())
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> crate::Result<u64> {
    Ok(0)
  }

  #[inline]
  async fn transaction(&mut self) -> crate::Result<Self::TransactionManager<'_>> {
    Ok(())
  }
}
