use crate::{
  database::client::postgres::{
    ExecutorBuffer, PostgresError, PostgresExecutor, message::MessageTy, protocol::query,
  },
  misc::{
    ConnectionState, LeaseMut, Stream, SuffixWriterFbvm,
    partitioned_filled_buffer::PartitionedFilledBuffer,
  },
};

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn simple_query_execute(
    cmd: &str,
    cs: &mut ConnectionState,
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
    mut cb: impl FnMut(u64) -> Result<(), E>,
  ) -> Result<(), E> {
    {
      let mut sw = SuffixWriterFbvm::from(net_buffer._suffix_writer());
      query(cmd.as_bytes(), &mut sw)?;
      stream.write_all(sw._curr_bytes()).await?;
    }
    loop {
      let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
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
