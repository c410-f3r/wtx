//! Database

use crate::database::{Database, FromRecord, RecordValues, TransactionManager};
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

  /// Executes a raw command returning the number of affected records.
  fn execute<E, RV>(&mut self, cmd: &str, rv: RV) -> impl Future<Output = Result<u64, E>>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>;

  /// Caches the passed command, speeding up subsequent calls that match the same `cmd`.
  ///
  /// Depending on the implementation, caching can be performed in the client or in the server.
  fn prepare(&mut self, cmd: &str) -> impl Future<Output = crate::Result<()>>;

  /// Retrieves a raw database record.
  fn record<E, SV>(
    &mut self,
    cmd: &str,
    sv: SV,
  ) -> impl Future<Output = Result<<Self::Database as Database>::Record<'_>, E>>
  where
    E: From<crate::Error>,
    SV: RecordValues<Self::Database, E>;

  /// Retrieves a set of raw database records.
  fn records<E, SV>(
    &mut self,
    cmd: &str,
    sv: SV,
    cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> impl Future<Output = Result<<Self::Database as Database>::Records<'_>, E>>
  where
    E: From<crate::Error>,
    SV: RecordValues<Self::Database, E>;

  /// Retrieves a record and maps it to `T`. See [FromRecord].
  #[inline]
  fn simple_entity<E, SV, T>(&mut self, cmd: &str, sv: SV) -> impl Future<Output = Result<T, E>>
  where
    E: From<crate::Error>,
    T: for<'rec> FromRecord<E, <Self::Database as Database>::Record<'rec>>,
    SV: RecordValues<Self::Database, E>,
  {
    async move { T::from_record(self.record(cmd, sv).await?) }
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
        .records(cmd, sv, |record| {
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
  async fn execute<E, RV>(&mut self, _: &str, _: RV) -> Result<u64, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
  {
    Ok(0)
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn record<E, SV>(
    &mut self,
    _: &str,
    _: SV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    E: From<crate::Error>,
    SV: RecordValues<Self::Database, E>,
  {
    Ok(())
  }

  #[inline]
  async fn records<E, SV>(
    &mut self,
    _: &str,
    _: SV,
    _: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    E: From<crate::Error>,
    SV: RecordValues<Self::Database, E>,
  {
    Ok(())
  }

  #[inline]
  async fn transaction(&mut self) -> crate::Result<Self::TransactionManager<'_>> {
    Ok(())
  }
}
