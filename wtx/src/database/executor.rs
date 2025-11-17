//! Database

use crate::{
  collection::TryExtend,
  database::{Database, DatabaseError, RecordValues, Records, StmtCmd},
  de::DEController,
  misc::ConnectionState,
};

/// A connection for executing database commands.
pub trait Executor {
  /// See [Database].
  type Database: Database;

  /// Sometimes the backend can discontinue the connection.
  fn connection_state(&self) -> ConnectionState;

  /// A version of [`Executor::execute_many`] where no records are expected.
  fn execute(
    &mut self,
    cmd: &str,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>> {
    async {
      let mut buffer = None;
      self.execute_many(&mut buffer, cmd, |_| Ok(())).await?;
      if buffer.is_some() {
        return Err(From::from(DatabaseError::UnexpectedRecords.into()));
      }
      Ok(())
    }
  }

  /// Allows the evaluation of severals commands separated by semicolons but nothing is cached
  /// or inspected for potential vulnerabilities.
  ///
  /// * Pass `&mut ()` to `buffer` if you don't want to populate returned values.
  /// * There are no statements, as such, returned values are treated as strings.
  fn execute_many<'this, B>(
    &'this mut self,
    buffer: &mut B,
    cmd: &str,
    cb: impl FnMut(
      <Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>
  where
    B: TryExtend<[<Self::Database as Database>::Records<'this>; 1]>;

  /// A version of [`Executor::execute_many`] where a single record is expected.
  fn execute_one(
    &mut self,
    cmd: &str,
  ) -> impl Future<
    Output = Result<
      <Self::Database as Database>::Record<'_>,
      <Self::Database as DEController>::Error,
    >,
  > {
    async {
      let mut buffer = None;
      self.execute_many(&mut buffer, cmd, |_| Ok(())).await?;
      let Some(records) = buffer else {
        return Err(From::from(DatabaseError::MissingSingleRecord.into()));
      };
      let (1, Some(record)) = (records.len(), records.get(0)) else {
        return Err(From::from(DatabaseError::MissingSingleRecord.into()));
      };
      Ok(record)
    }
  }

  /// A version of [`Executor::execute_with_stmt_many`] where no records are expected.
  fn execute_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    async {
      let records = self.execute_with_stmt_many(sc, rv, |_| Ok(())).await?;
      if records.len() > 0 {
        Err(From::from(DatabaseError::UnexpectedRecords.into()))
      } else {
        Ok(())
      }
    }
  }

  /// Executes a **single** statement automatically binding the values of `rv`. Expects and
  /// returns an arbitrary number of records.
  fn execute_with_stmt_many<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
    cb: impl FnMut(
      <Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> impl Future<
    Output = Result<
      <Self::Database as Database>::Records<'_>,
      <Self::Database as DEController>::Error,
    >,
  >
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd;

  /// A version of [`Executor::execute_with_stmt_many`] where a single record is expected.
  fn execute_with_stmt_one<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> impl Future<
    Output = Result<
      <Self::Database as Database>::Record<'_>,
      <Self::Database as DEController>::Error,
    >,
  >
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    async {
      let records = self.execute_with_stmt_many(sc, rv, |_| Ok(())).await?;
      let (1, Some(record)) = (records.len(), records.get(0)) else {
        return Err(From::from(DatabaseError::MissingSingleRecord.into()));
      };
      Ok(record)
    }
  }

  /// Pings the server to signal an active connection
  fn ping(&mut self) -> impl Future<Output = Result<(), <Self::Database as DEController>::Error>>;

  /// Caches the passed command to create a statement, which speeds up subsequent calls that match
  /// the same `cmd`.
  ///
  /// The returned integer is an identifier of the added statement.
  fn prepare(
    &mut self,
    cmd: &str,
  ) -> impl Future<Output = Result<u64, <Self::Database as DEController>::Error>>;

  /// Makes internal calls to "BEGIN" and "COMMIT".
  fn transaction<'this, F, R>(
    &'this mut self,
    fun: impl FnOnce(&'this mut Self) -> F,
  ) -> impl Future<Output = Result<R, <Self::Database as DEController>::Error>>
  where
    F: Future<Output = Result<(R, &'this mut Self), <Self::Database as DEController>::Error>>,
  {
    async move {
      self.execute_many(&mut (), "BEGIN", |_| Ok(())).await?;
      let (rslt, this) = fun(self).await?;
      this.execute_many(&mut (), "COMMIT", |_| Ok(())).await?;
      Ok(rslt)
    }
  }
}

impl<T> Executor for &mut T
where
  T: Executor,
{
  type Database = T::Database;

  #[inline]
  fn connection_state(&self) -> ConnectionState {
    (**self).connection_state()
  }

  #[inline]
  async fn execute_many<'this, B>(
    &'this mut self,
    buffer: &mut B,
    cmd: &str,
    cb: impl FnMut(
      <Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> Result<(), <Self::Database as DEController>::Error>
  where
    B: TryExtend<[<Self::Database as Database>::Records<'this>; 1]>,
  {
    (**self).execute_many(buffer, cmd, cb).await
  }

  #[inline]
  async fn execute_with_stmt_many<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
    cb: impl FnMut(
      <Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> Result<<Self::Database as Database>::Records<'_>, <Self::Database as DEController>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    (**self).execute_with_stmt_many(sc, rv, cb).await
  }

  #[inline]
  async fn ping(&mut self) -> Result<(), <Self::Database as DEController>::Error> {
    (**self).ping().await
  }

  #[inline]
  async fn prepare(&mut self, cmd: &str) -> Result<u64, <Self::Database as DEController>::Error> {
    (**self).prepare(cmd).await
  }
}

impl Executor for () {
  type Database = ();

  #[inline]
  fn connection_state(&self) -> ConnectionState {
    ConnectionState::Closed
  }

  #[inline]
  async fn execute_many<'this, B>(
    &'this mut self,
    _: &mut B,
    _: &str,
    _: impl FnMut(
      <Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> Result<(), <Self::Database as DEController>::Error>
  where
    B: TryExtend<[<Self::Database as Database>::Records<'this>; 1]>,
  {
    Ok(())
  }

  #[inline]
  async fn execute_with_stmt_many<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
    _: impl FnMut(
      <Self::Database as Database>::Record<'_>,
    ) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> Result<<Self::Database as Database>::Records<'_>, <Self::Database as DEController>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    Ok(())
  }

  #[inline]
  async fn ping(&mut self) -> Result<(), <Self::Database as DEController>::Error> {
    Ok(())
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> Result<u64, <Self::Database as DEController>::Error> {
    Ok(0)
  }
}
