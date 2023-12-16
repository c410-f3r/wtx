use crate::{
  database::client::postgres::{Executor, ExecutorBuffer},
  misc::Stream,
};
use core::borrow::BorrowMut;

/// Transaction Manager
#[derive(Debug)]
pub struct TransactionManager<'exec, EB, S> {
  executor: &'exec mut Executor<EB, S>,
}

impl<'exec, EB, S> TransactionManager<'exec, EB, S> {
  pub(crate) fn new(executor: &'exec mut Executor<EB, S>) -> Self {
    Self { executor }
  }
}

impl<'exec, EB, S> crate::database::TransactionManager for TransactionManager<'exec, EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  type Executor = Executor<EB, S>;

  #[inline]
  async fn begin(&mut self) -> crate::Result<()> {
    self.executor.simple_query_execute("BEGIN", |_| {}).await?;
    Ok(())
  }

  #[inline]
  async fn commit(self) -> crate::Result<()> {
    self.executor.simple_query_execute("COMMIT", |_| {}).await?;
    Ok(())
  }

  #[inline]
  fn executor(&mut self) -> &mut Self::Executor {
    self.executor
  }
}
