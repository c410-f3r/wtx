use crate::{
  collection::{TryExtend, Vector},
  database::{
    StmtCmd,
    client::{
      postgres::{
        ExecutorBuffer, PostgresError, PostgresExecutor, PostgresRecord, PostgresRecords,
        PostgresStatements, Ty, message::MessageTy, misc::dummy_stmt_value, msg_field::MsgField,
        postgres_column_info::PostgresColumnInfo, protocol::query,
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
    let mut curr_stmt_idx = 0;
    let mut values_params_offset = 0;
    loop {
      let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(rows_len) => {
          if !B::IS_UNIT {
            let Some(stmt) = stmts.get_by_idx_mut(curr_stmt_idx) else {
              return Err(crate::Error::ProgrammingError.into());
            };
            *stmt.rows_len = *Usize::from(rows_len);
            values_params_offset = values_params.len();
          }
        }
        MessageTy::DataRow(values_len) => {
          if !B::IS_UNIT {
            let Some(stmt_mut) = stmts.get_by_idx_mut(curr_stmt_idx) else {
              return Err(crate::Error::ProgrammingError.into());
            };
            let net_buffer_range = begin_data..net_buffer.current_end_idx();
            let mut bytes = net_buffer.all().get(net_buffer_range).unwrap_or_default();
            let record_range_begin = net_buffer.antecedent_end_idx().wrapping_sub(begin);
            let record_range_end = net_buffer.current_end_idx().wrapping_sub(begin_data);
            bytes = bytes.get(record_range_begin..record_range_end).unwrap_or_default();
            let values_params_begin = values_params.len().wrapping_sub(values_params_offset);
            cb(PostgresRecord::parse(bytes, stmt_mut.stmt(), values_len, values_params)?)?;
            records_params.push((
              record_range_begin..record_range_end,
              values_params_begin..values_params.len().wrapping_sub(values_params_offset),
            ))?;
          }
        }
        MessageTy::EmptyQueryResponse => {}
        MessageTy::ReadyForQuery => break,
        MessageTy::RowDescription(columns_len, mut rd) => {
          if !B::IS_UNIT {
            let timestamp_nanos_str = timestamp_nanos_str()?;
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
            let elements = builder.inserted_elements();
            for idx in 0..columns_len {
              let (read, msg_field) = MsgField::parse(rd)?;
              let ty = Ty::Custom(msg_field.type_oid);
              let Some(element) = elements.get_mut(usize::from(idx)) else {
                break;
              };
              element.0 = PostgresColumnInfo::new(msg_field.name.try_into()?, ty);
              if let Some(elem @ [_not_empty, ..]) = rd.get(read..) {
                rd = elem;
              } else {
                break;
              }
            }
            let sm = StatementsMisc::new(timestamp_nanos_str, columns_len.into(), 0, 0);
            curr_stmt_idx = builder.build(stmt_cmd_id, sm)?;
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
    if !B::IS_UNIT {
      let mut rows_idx: usize = 0;
      let mut values_idx: usize = 0;
      for idx in stmts_begin..stmts.len() {
        let Some(stmt) = stmts.get_by_idx(idx) else {
          return Err(crate::Error::ProgrammingError.into());
        };
        let local_rows_idx = rows_idx.wrapping_add(stmt.rows_len);
        let local_values_idx = stmt.columns_len.wrapping_mul(local_rows_idx);
        let local_rp = records_params.get(rows_idx..local_rows_idx).unwrap_or_default();
        let local_vp = values_params.get(values_idx..local_values_idx).unwrap_or_default();
        rows_idx = local_rows_idx;
        values_idx = local_values_idx;
        buffer.try_extend([PostgresRecords::new(
          net_buffer.all().get(begin_data..net_buffer.current_end_idx()).unwrap_or_default(),
          local_rp,
          stmt,
          local_vp,
        )])?;
      }
    }
    Ok(())
  }
}
