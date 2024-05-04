use crate::{
  database::client::postgres::{Executor, ExecutorBuffer},
  misc::{LeaseMut, Stream},
};

/// Transaction Manager
#[derive(Debug)]
pub struct TransactionManager<'exec, E, EB, S> {
  executor: &'exec mut Executor<E, EB, S>,
}

impl<'exec, E, EB, S> TransactionManager<'exec, E, EB, S> {
  pub(crate) fn new(executor: &'exec mut Executor<E, EB, S>) -> Self {
    Self { executor }
  }
}

impl<'exec, E, EB, S> crate::database::TransactionManager for TransactionManager<'exec, E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  type Executor = Executor<E, EB, S>;

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
