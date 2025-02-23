use crate::{
  database::client::postgres::{
    ExecutorBuffer, PostgresError, PostgresExecutor, executor_buffer::ExecutorBufferPartsMut,
    message::MessageTy, protocol::query,
  },
  misc::{LeaseMut, Stream, SuffixWriterFbvm},
};

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn simple_query_execute(
    &mut self,
    cmd: &str,
    mut cb: impl FnMut(u64) -> Result<(), E>,
  ) -> Result<(), E> {
    {
      let ExecutorBufferPartsMut { nb, rb, vb, .. } = self.eb.lease_mut().parts_mut();
      ExecutorBuffer::clear_cmd_buffers(nb, rb, vb);
      let mut sw = SuffixWriterFbvm::from(self.eb.lease_mut().nb._suffix_writer());
      query(cmd.as_bytes(), &mut sw)?;
      self.stream.write_all(sw._curr_bytes()).await?;
    }
    loop {
      let nb = &mut self.eb.lease_mut().nb;
      let msg = Self::fetch_msg_from_stream(&mut self.cs, nb, &mut self.stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(n) => cb(n)?,
        MessageTy::EmptyQueryResponse => {
          cb(0)?;
        }
        MessageTy::ReadyForQuery => return Ok(()),
        _ => {
          return Err(
            crate::Error::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag })
              .into(),
          );
        }
      }
    }
  }
}
