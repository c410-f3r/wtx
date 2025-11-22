use crate::{
  collection::{ArrayVector, ArrayVectorU8, TryExtend},
  database::{
    Database, DatabaseError, RecordValues, StmtCmd,
    client::{
      postgres::{
        Postgres, PostgresError, PostgresExecutor, PostgresRecord,
        executor_buffer::ExecutorBuffer,
        message::MessageTy,
        misc::{data_row, extend_records, row_description},
        protocol::sync,
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
  de::u64_string,
  misc::{LeaseMut, SuffixWriterFbvm, Usize},
  stream::Stream,
};

/// Sends multiple statements at once and awaits them when flushed.
#[derive(Debug)]
pub struct Batch<'exec, E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
{
  executor: &'exec mut PostgresExecutor<E, EB, S>,
  len: usize,
  stmt_cmd_ids: ArrayVectorU8<u64, 8>,
}

impl<'exec, E, EB, S> Batch<'exec, E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) fn new(executor: &'exec mut PostgresExecutor<E, EB, S>) -> Self {
    let PostgresExecutor { eb, .. } = executor;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, values_params, .. } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Self { executor, len: 0, stmt_cmd_ids: ArrayVector::new() }
  }

  /// Combines received results based on previous [`Self::stmt`] calls.
  #[inline]
  pub async fn flush<B>(
    mut self,
    buffer: &mut B,
    mut cb: impl FnMut(PostgresRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[<Postgres<E> as Database>::Records<'exec>; 1]>,
  {
    let PostgresExecutor { cs, eb, phantom: _, stream } = self.executor;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params, .. } = common;
    {
      let mut sw = net_buffer.suffix_writer();
      sync(&mut sw)?;
      stream.write_all(sw.all_bytes()).await?;
    }

    let begin = net_buffer.current_end_idx();
    let begin_data = net_buffer.current_end_idx().wrapping_add(7);
    let stmts_begin = stmts.len();
    let mut stmt_cmd_ids_iter = self.stmt_cmd_ids.into_iter();
    let mut values_params_offset = 0;
    'stmts: loop {
      let Some(stmt_cmd_id) = stmt_cmd_ids_iter.next() else {
        let msg =
          PostgresExecutor::<E, EB, S>::fetch_msg_from_stream(cs, net_buffer, stream).await?;
        if let MessageTy::ReadyForQuery = msg.ty {
          break 'stmts;
        } else {
          return Err(crate::Error::ProgrammingError.into());
        }
      };
      let stmt_mut = PostgresExecutor::<E, EB, S>::await_stmt_prepare::<false>(
        cs,
        net_buffer,
        stmt_cmd_id,
        u64_string(stmt_cmd_id),
        stmts,
        stream,
      )
      .await?;
      PostgresExecutor::<E, EB, S>::await_stmt_bind(cs, net_buffer, stream).await?;

      'rows: loop {
        let msg =
          PostgresExecutor::<E, EB, S>::fetch_msg_from_stream(cs, net_buffer, stream).await?;
        match msg.ty {
          MessageTy::CommandComplete(rows_len) => {
            if !B::IS_UNIT {
              *stmt_mut.rows_len = *Usize::from(rows_len);
              values_params_offset = values_params.len();
            }
            break 'rows;
          }
          MessageTy::DataRow(values_len) => {
            if !B::IS_UNIT {
              data_row(
                begin,
                begin_data,
                net_buffer,
                records_params,
                stmt_mut.stmt(),
                values_len,
                values_params,
                values_params_offset,
                &mut cb,
              )?;
            }
          }
          MessageTy::EmptyQueryResponse => {}
          MessageTy::ReadyForQuery => break 'stmts,
          MessageTy::RowDescription(columns_len, mut rd) => {
            if !B::IS_UNIT {
              row_description(columns_len, &mut rd, |_, _| Ok(()))?;
            }
          }
          _ => {
            return Err(
              crate::Error::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag })
                .into(),
            );
          }
        }
      }
    }
    extend_records(
      begin_data,
      buffer,
      net_buffer,
      records_params,
      stmts,
      stmts_begin,
      values_params,
    )?;
    self.len = 0;
    Ok(())
  }

  /// Pushes a single statement for a later submission.
  #[inline]
  pub fn stmt<SC, RV>(&mut self, sc: SC, rv: RV) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
    SC: StmtCmd,
  {
    let PostgresExecutor { eb, .. } = self.executor;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, stmts, .. } = common;
    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    let len = {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      PostgresExecutor::<E, EB, S>::write_stmt_prepare::<false>(
        stmt_cmd,
        &stmt_cmd_id_array,
        &mut sw,
        &[],
      )?;
      PostgresExecutor::<E, EB, S>::write_stmt_bind::<_, false>(rv, &stmt_cmd_id_array, &mut sw)?;
      self.len = self.len.wrapping_add(1);
      sw.curr_bytes().len()
    };
    net_buffer.set_indices(0, net_buffer.current().len().wrapping_add(len), 0)?;
    self.stmt_cmd_ids.push(stmt_cmd_id)?;
    Ok(())
  }
}
