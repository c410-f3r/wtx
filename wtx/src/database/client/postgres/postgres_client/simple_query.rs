use crate::{
  codec::{U64String, u64_string},
  collections::{TryExtend, Vector},
  database::{
    StmtCmd as _,
    client::{
      postgres::{
        PostgresClient, PostgresError, PostgresRecord, PostgresRecords, PostgresStatements,
        message::MessageTy,
        misc::{data_row, dummy_stmt_value, extend_records, row_description},
        protocol::query,
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  misc::{Either, Usize},
  net::{BufStreamReader, ConnectionState, Stream, StreamWriter as _},
  sync::AtomicU64,
  tls::{TlsMode, TlsStream},
};
use core::{ops::Range, sync::atomic::Ordering};

impl<E, S, TM> PostgresClient<E, S, TM>
where
  E: From<crate::Error>,
  S: Stream,
  TM: TlsMode,
{
  #[expect(clippy::wildcard_enum_match_arm, reason = "too many variants")]
  pub(crate) async fn simple_query_execute<'exec, B>(
    buffer: &mut B,
    cmd: &str,
    cs: &mut ConnectionState,
    read_buffer: &'exec mut BufStreamReader,
    records_params: &'exec mut Vector<(Range<usize>, Range<usize>)>,
    stmts: &'exec mut PostgresStatements,
    stream: &mut TlsStream<S, TM, true>,
    values_params: &'exec mut Vector<(bool, Range<usize>)>,
    mut cb: impl FnMut(PostgresRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[PostgresRecords<'exec, E>; 1]>,
  {
    {
      let mut sw = read_buffer.suffix_pusher();
      query(cmd.as_bytes(), sw.inner_mut())?;
      stream.write_all(sw.curr()).await?;
    }
    let begin_data = read_buffer.current_end_idx().wrapping_add(7);
    let stmts_begin = stmts.len();
    let mut stmt_idx = None;
    let mut values_params_offset = 0;
    *read_buffer.forbid_clear_mut() = true;
    loop {
      let msg = Self::fetch_msg(cs, read_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(rows_len) => {
          if !B::IS_UNIT {
            if let Some(stmt) = stmt_idx.and_then(|idx| stmts.get_by_idx_mut(idx)) {
              *stmt.rows_len = *Usize::from(rows_len);
            }
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
              begin_data,
              read_buffer,
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
            static ID: AtomicU64 = AtomicU64::new(1);
            let timestamp_nanos_str = u64_string(ID.fetch_add(1, Ordering::Relaxed));
            let stmt_cmd_id = timestamp_nanos_str.as_str().hash(stmts.hasher_mut());
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
              StatementsMisc::new(timestamp_nanos_str, columns_len.into(), 0, 0),
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
    *read_buffer.forbid_clear_mut() = false;
    extend_records(
      begin_data,
      buffer,
      read_buffer,
      records_params,
      stmts,
      (stmts_begin..stmts.len()).map(Either::Left),
      values_params,
    )?;
    Ok(())
  }
}
