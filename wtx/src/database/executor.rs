//! Database

use crate::{
  database::{Database, FromRecord, RecordValues, StmtCmd},
  misc::ConnectionState,
};
use core::future::Future;

/// A connection for executing database commands.
pub trait Executor {
  /// See [Database].
  type Database: Database;

  /// Sometimes the backend can discontinue the connection.
  fn connection_state(&self) -> ConnectionState;

  /// Allows the evaluation of severals commands returning the number of affected records on each `cb` call.
  ///
  /// Commands are not cached or inspected for potential vulnerabilities.
  fn execute(&mut self, cmd: &str, cb: impl FnMut(u64)) -> impl Future<Output = crate::Result<()>>;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt` and then returns the number of affected records.
  fn execute_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> impl Future<Output = Result<u64, <Self::Database as Database>::Error>>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt` and then returns a **single** record.
  fn fetch_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> impl Future<
    Output = Result<<Self::Database as Database>::Record<'_>, <Self::Database as Database>::Error>,
  >
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd;

  /// Executes a **single** statement automatically binding the values of `rv` to the referenced
  /// `stmt` and then returns a **set** of records.
  fn fetch_many_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
    cb: impl FnMut(
      &<Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as Database>::Error>,
  ) -> impl Future<
    Output = Result<<Self::Database as Database>::Records<'_>, <Self::Database as Database>::Error>,
  >
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd;

  /// Caches the passed command to create a statement, which speeds up subsequent calls that match
  /// the same `cmd`.
  ///
  /// The returned integer is an identifier of the added statement.
  fn prepare(
    &mut self,
    cmd: &str,
  ) -> impl Future<Output = Result<u64, <Self::Database as Database>::Error>>;

  /// Retrieves a record and maps it to `T`. See [`FromRecord`].
  #[inline]
  fn simple_entity<SV, T>(
    &mut self,
    cmd: &str,
    sv: SV,
  ) -> impl Future<Output = Result<T, <Self::Database as Database>::Error>>
  where
    T: FromRecord<Self::Database>,
    SV: RecordValues<Self::Database>,
  {
    async move { T::from_record(&self.fetch_with_stmt(cmd, sv).await?) }
  }

  /// Retrieves a set of records and maps them to the corresponding `T`. See [`FromRecord`].
  #[inline]
  fn simple_entities<SV, T>(
    &mut self,
    cmd: &str,
    sv: SV,
    mut cb: impl FnMut(T) -> Result<(), <Self::Database as Database>::Error>,
  ) -> impl Future<Output = Result<(), <Self::Database as Database>::Error>>
  where
    SV: RecordValues<Self::Database>,
    T: FromRecord<Self::Database>,
  {
    async move {
      let _rec = self.fetch_many_with_stmt(cmd, sv, |record| cb(T::from_record(record)?)).await?;
      Ok(())
    }
  }

  /// Makes internal calls to "BEGIN" and "COMMIT".
  fn transaction<'this, F, R>(
    &'this mut self,
    fun: impl FnOnce(&'this mut Self) -> F,
  ) -> impl Future<Output = Result<R, <Self::Database as Database>::Error>>
  where
    F: Future<Output = Result<(R, &'this mut Self), <Self::Database as Database>::Error>>,
  {
    async move {
      self.execute("BEGIN", |_| {}).await?;
      let (rslt, this) = fun(self).await?;
      this.execute("COMMIT", |_| {}).await?;
      Ok(rslt)
    }
  }
}

impl Executor for () {
  type Database = ();

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
    _: impl FnMut(
      &<Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as Database>::Error>,
  ) -> Result<(), <Self::Database as Database>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    Ok(())
  }

  #[inline]
  fn connection_state(&self) -> ConnectionState {
    ConnectionState::Closed
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> Result<u64, <Self::Database as Database>::Error> {
    Ok(0)
  }
}
