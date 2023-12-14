use crate::{
  database::client::postgres::{query, Executor, ExecutorBuffer, MessageTy},
  misc::{FilledBufferWriter, Stream},
};
use core::borrow::BorrowMut;

impl<EB, S> Executor<EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn query_ignored(&mut self, cmd: &str) -> crate::Result<()> {
    let mut fbw = FilledBufferWriter::from(&mut self.eb.borrow_mut().nb);
    query(cmd.as_bytes(), &mut fbw)?;
    self.stream.write_all(fbw._curr_bytes()).await?;
    let msg = Self::fetch_msg_from_stream(&mut self.eb.borrow_mut().nb, &mut self.stream).await?;
    match msg.ty {
      MessageTy::ReadyForQuery => Ok(()),
      _ => Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag }),
    }
  }
}
