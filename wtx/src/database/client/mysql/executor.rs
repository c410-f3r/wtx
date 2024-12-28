use crate::{
  database::{
    client::mysql::{ExecutorBuffer, Mysql, TransactionManager},
    Database, RecordValues, StmtCmd,
  },
  misc::{ConnectionState, DEController, LeaseMut, Stream},
};
use core::marker::PhantomData;

/// Executor
#[derive(Debug)]
pub struct Executor<E, EB, S> {
  pub(crate) cs: ConnectionState,
  pub(crate) eb: EB,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stream: S,
}

impl<E, EB, S> Executor<E, EB, S> {
  #[inline]
  pub async fn connect(eb: EB, stream: S) -> crate::Result<Self> {
    Ok(Self { cs: ConnectionState::Open, eb, phantom: PhantomData, stream })
  }
}

impl<E, EB, S> crate::database::Executor for Executor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  type Database = Mysql<E>;
  type TransactionManager<'tm>
    = TransactionManager<'tm, E, EB, S>
  where
    Self: 'tm;

  #[inline]
  fn connection_state(&self) -> ConnectionState {
    self.cs
  }

  #[inline]
  async fn execute(&mut self, _: &str, _: impl FnMut(u64)) -> crate::Result<()> {
    todo!()
  }

  #[inline]
  async fn execute_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
  ) -> Result<u64, <Self::Database as DEController>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    todo!()
  }

  #[inline]
  async fn fetch_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    todo!()
  }

  #[inline]
  async fn fetch_many_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
    _: impl FnMut(&<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<<Self::Database as Database>::Records<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    todo!()
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> Result<u64, E> {
    todo!()
  }

  #[inline]
  async fn transaction(&mut self) -> crate::Result<Self::TransactionManager<'_>> {
    todo!()
  }
}
