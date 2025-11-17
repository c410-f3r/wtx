use crate::{
  collection::TryExtend,
  database::{
    Database, DatabaseError, RecordValues, StmtCmd,
    client::{
      postgres::{
        Postgres, PostgresExecutor, executor_buffer::ExecutorBuffer,
        postgres_executor::commons::FetchWithStmtCommons, protocol::sync,
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
  de::u64_string,
  misc::{LeaseMut, SuffixWriterFbvm},
  stream::Stream,
};

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  /// Combines received results based on previous [`Self::batch_stmt`] calls.
  ///
  /// Does nothing if [`Self::batch_stmt`] wasn't called before.
  ///
  /// * Use [`Executor::execute`] if you know what you are doing.
  /// * Use methods with the `_stmt` suffix if you want to send a single statement.
  #[inline]
  pub async fn batch_flush<'this, B>(&'this mut self, _: &mut B) -> Result<(), E>
  where
    B: TryExtend<[<Postgres<E> as Database>::Records<'this>; 1]>,
  {
    let Self { cs: _, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, plen, .. } = common;
    let mut sw = net_buffer.suffix_writer();
    sync(&mut sw)?;
    stream.write_all(sw.all_bytes()).await?;
    *plen = 0;
    Ok(())
  }

  /// Pushes a statement for a single subsequent submission.
  ///
  /// Any other call, taking aside [`Self::batch_flush`], will erase the contents created by this
  /// method.
  ///
  /// * Use [`Executor::execute`] if you know what you are doing.
  /// * Use methods with the `_stmt` suffix if you want to send a single statement.
  #[inline]
  pub fn batch_stmt<SC, RV>(&mut self, sc: SC, rv: RV) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
    SC: StmtCmd,
  {
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, plen, records_params, stmts, values_params } = common;
    if *plen == 0 {
      clear_cmd_buffers(net_buffer, records_params, values_params);
    }
    let mut fwsc = FetchWithStmtCommons { cs, stream, tys: &[] };
    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    let len = {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      Self::write_stmt_parse::<false>(&mut fwsc, stmt_cmd, &stmt_cmd_id_array, &mut sw)?;
      Self::write_stmt_rest::<_, false>(rv, &stmt_cmd_id_array, &mut sw)?;
      *plen = plen.wrapping_add(1);
      sw.curr_bytes().len()
    };
    net_buffer.set_indices(0, net_buffer.current().len().wrapping_add(len), 0)?;
    Ok(())
  }
}
