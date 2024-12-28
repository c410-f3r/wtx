use crate::{
  database::client::mysql::{Executor, ExecutorBuffer},
  misc::{LeaseMut, Stream},
};

/// Transaction Manager
#[derive(Debug)]
pub struct TransactionManager<'exec, E, EB, S> {
  executor: &'exec mut Executor<E, EB, S>,
}

impl<'exec, E, EB, S> crate::database::TransactionManager for TransactionManager<'exec, E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  type Executor = Executor<E, EB, S>;

  #[inline]
  async fn begin(&mut self) -> crate::Result<()> {
    todo!()
  }

  #[inline]
  async fn commit(self) -> crate::Result<()> {
    todo!()
  }

  #[inline]
  fn executor(&mut self) -> &mut Self::Executor {
    self.executor
  }
}
