use crate::{
  collection::{TryExtend, Vector},
  database::{
    StmtCmd,
    client::{
      postgres::{
        ExecutorBuffer, PostgresError, PostgresExecutor, PostgresRecord, PostgresRecords,
        PostgresStatements,
        message::MessageTy,
        misc::{data_row, dummy_stmt_value, extend_records, row_description},
        protocol::query,
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  de::U64String,
  misc::{
    ConnectionState, LeaseMut, SuffixWriterFbvm, Usize, net::PartitionedFilledBuffer,
    timestamp_nanos_str,
  },
  stream::Stream,
};
use core::ops::Range;

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn simple_query_execute<'exec, B>(
    buffer: &mut B,
    cmd: &str,
    cs: &mut ConnectionState,
    net_buffer: &'exec mut PartitionedFilledBuffer,
    records_params: &'exec mut Vector<(Range<usize>, Range<usize>)>,
    stmts: &'exec mut PostgresStatements,
    stream: &mut S,
    values_params: &'exec mut Vector<(bool, Range<usize>)>,
    mut cb: impl FnMut(PostgresRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[PostgresRecords<'exec, E>; 1]>,
  {
    {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      query(cmd.as_bytes(), &mut sw)?;
      stream.write_all(sw.curr_bytes()).await?;
    }
    let begin = net_buffer.current_end_idx();
    let begin_data = net_buffer.current_end_idx().wrapping_add(7);
    let stmts_begin = stmts.len();
    let mut stmt_idx = None;
    let mut values_params_offset = 0;
    loop {
      let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(rows_len) => {
          if !B::IS_UNIT {
            if let Some(stmt) = stmt_idx.and_then(|idx| stmts.get_by_idx_mut(idx)) {
              *stmt.rows_len = *Usize::from(rows_len);
            };
            values_params_offset = values_params.len();
          }
          stmt_idx = None;
        }
        MessageTy::DataRow(values_len) => {
          if !B::IS_UNIT {
            let Some(stmt_mut) = stmt_idx.and_then(|idx| stmts.get_by_idx_mut(idx)) else {
              return Err(crate::Error::ProgrammingError.into());
            };
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
        MessageTy::ReadyForQuery => break,
        MessageTy::RowDescription(columns_len, mut rd) => {
          if !B::IS_UNIT {
            let timestamp_nanos_str = timestamp_nanos_str()?;
            let stmt_cmd_id = timestamp_nanos_str.1.as_str().hash(stmts.hasher_mut());
            let mut builder = stmts
              .builder((), {
                async fn fun(_: &mut (), _: StatementsMisc<U64String>) -> crate::Result<()> {
                  Ok(())
                }
                fun
              })
              .await?;
            let _ = builder.expand(columns_len.into(), dummy_stmt_value())?;
            stmt_idx = Some(builder.build(
              stmt_cmd_id,
              StatementsMisc::new(timestamp_nanos_str.1, columns_len.into(), 0, 0),
            )?);
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
    extend_records(
      begin_data,
      buffer,
      net_buffer,
      records_params,
      stmts,
      stmts_begin,
      values_params,
    )?;
    Ok(())
  }
}
