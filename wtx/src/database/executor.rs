//! Database

use crate::{
  database::{Database, FromRecord, RecordValues, StmtCmd, TransactionManager},
  misc::AsyncBounds,
};
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

  /// Allows the evaluation of severals commands returning the number of affected records on each `cb` call.
  ///
  /// Commands are not cached or inspected for potential vulnerabilities.
  fn execute(
    &mut self,
    cmd: &str,
    cb: impl AsyncBounds + FnMut(u64),
  ) -> impl AsyncBounds + Future<Output = crate::Result<()>>;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt` and then returns the number of affected records.
  fn execute_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> impl AsyncBounds + Future<Output = Result<u64, <Self::Database as Database>::Error>>
  where
    RV: AsyncBounds + RecordValues<Self::Database>,
    SC: AsyncBounds + StmtCmd;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt` and then returns a **single** record.
  fn fetch_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    sv: RV,
  ) -> impl AsyncBounds
       + Future<
    Output = Result<<Self::Database as Database>::Record<'_>, <Self::Database as Database>::Error>,
  >
  where
    RV: AsyncBounds + RecordValues<Self::Database>,
    SC: AsyncBounds + StmtCmd;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt` and then returns a **set** of records.
  fn fetch_many_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    sv: RV,
    cb: impl AsyncBounds
      + FnMut(
        &<Self::Database as Database>::Record<'_>,
      ) -> Result<(), <Self::Database as Database>::Error>,
  ) -> impl AsyncBounds
       + Future<
    Output = Result<<Self::Database as Database>::Records<'_>, <Self::Database as Database>::Error>,
  >
  where
    RV: AsyncBounds + RecordValues<Self::Database>,
    SC: AsyncBounds + StmtCmd;

  /// Sometimes the backend can discontinue the connection.
  fn is_closed(&self) -> bool;

  /// Caches the passed command to create a statement, which speeds up subsequent calls that match
  /// the same `cmd`.
  ///
  /// The returned integer is an identifier of the added statement.
  fn prepare(
    &mut self,
    cmd: &str,
  ) -> impl AsyncBounds + Future<Output = Result<u64, <Self::Database as Database>::Error>>;

  /// Retrieves a record and maps it to `T`. See [FromRecord].
  #[inline]
  fn simple_entity<SV, T>(
    &mut self,
    cmd: &str,
    sv: SV,
  ) -> impl AsyncBounds + Future<Output = Result<T, <Self::Database as Database>::Error>>
  where
    T: AsyncBounds + FromRecord<Self::Database>,
    SV: AsyncBounds + RecordValues<Self::Database>,
    for<'any> &'any mut Self: AsyncBounds,
  {
    async move { T::from_record(&self.fetch_with_stmt(cmd, sv).await?) }
  }

  /// Retrieves a set of records and maps them to the corresponding `T`. See [FromRecord].
  #[inline]
  fn simple_entities<SV, T>(
    &mut self,
    cmd: &str,
    results: &mut Vec<T>,
    sv: SV,
  ) -> impl AsyncBounds + Future<Output = Result<(), <Self::Database as Database>::Error>>
  where
    SV: AsyncBounds + RecordValues<Self::Database>,
    T: AsyncBounds + FromRecord<Self::Database>,
    for<'any> &'any mut Self: AsyncBounds,
  {
    async move {
      let _records = self
        .fetch_many_with_stmt(cmd, sv, |record| {
          results.push(T::from_record(record)?);
          Ok(())
        })
        .await?;
      Ok(())
    }
  }

  /// Initially calls `begin` and the returns [Self::TransactionManager], which implies in an
  /// following mandatory `commit` call by the caller.
  fn transaction(
    &mut self,
  ) -> impl AsyncBounds + Future<Output = crate::Result<Self::TransactionManager<'_>>>;
}

impl Executor for () {
  type Database = ();
  type TransactionManager<'tm> = ();

  #[inline]
  async fn execute(&mut self, _: &str, _: impl FnMut(u64)) -> crate::Result<()> {
    Ok(())
  }

  #[inline]
  async fn execute_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
  ) -> Result<u64, <Self::Database as Database>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    Ok(0)
  }

  #[inline]
  async fn fetch_with_stmt<S, RV>(
    &mut self,
    _: S,
    _: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, <Self::Database as Database>::Error>
  where
    RV: RecordValues<Self::Database>,
    S: StmtCmd,
  {
    Ok(())
  }

  #[inline]
  async fn fetch_many_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
    _: impl AsyncBounds
      + FnMut(
        &<Self::Database as Database>::Record<'_>,
      ) -> Result<(), <Self::Database as Database>::Error>,
  ) -> Result<(), <Self::Database as Database>::Error>
  where
    RV: AsyncBounds + RecordValues<Self::Database>,
    SC: AsyncBounds + StmtCmd,
  {
    Ok(())
  }

  #[inline]
  fn is_closed(&self) -> bool {
    true
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> Result<u64, <Self::Database as Database>::Error> {
    Ok(0)
  }

  #[inline]
  async fn transaction(&mut self) -> crate::Result<Self::TransactionManager<'_>> {
    Ok(())
  }
}
