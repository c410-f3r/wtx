use crate::{
  collection::Vector,
  database::{
    DatabaseError, StmtCmd,
    client::{
      mysql::{
        ExecutorBuffer, MysqlExecutor, MysqlStatementMut, MysqlStatements,
        misc::{dummy_stmt_value, fetch_protocol, send_packet},
        mysql_column_info::MysqlColumnInfo,
        protocol::{
          column_res::ColumnRes, prepare_req::PrepareReq, prepare_res::PrepareRes,
          stmt_close_req::StmtCloseReq,
        },
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  de::{U64String, u64_string},
  misc::{LeaseMut, net::PartitionedFilledBuffer},
  stream::Stream,
};

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn write_send_await_stmt<'stmts, SC>(
    (capabilities, sequence_id): (&mut u64, &mut u8),
    encode_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut MysqlStatements,
    stream: &mut S,
    tys_len: usize,
  ) -> Result<(u64, U64String, MysqlStatementMut<'stmts>), E>
  where
    SC: StmtCmd,
  {
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    if stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).is_some() {
      // FIXME(stable): Use `if let Some ...` with polonius
      let stmt_mut = stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).unwrap();
      return Ok((stmt_cmd_id, stmt_cmd_id_array, stmt_mut));
    }

    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;

    send_packet(
      (capabilities, sequence_id),
      encode_buffer,
      PrepareReq { query: stmt_cmd.as_bytes() },
      stream,
    )
    .await?;
    let mut builder = stmts
      .builder(((&mut *capabilities, &mut *sequence_id), encode_buffer, &mut *stream), {
        async fn fun<S>(
          ((capabilities, sequence_id), encode_buffer, stream): &mut (
            (&mut u64, &mut u8),
            &mut Vector<u8>,
            &mut S,
          ),
          stmt: StatementsMisc<u32>,
        ) -> crate::Result<()>
        where
          S: Stream,
        {
          send_packet(
            (capabilities, sequence_id),
            encode_buffer,
            StmtCloseReq { statement: stmt._aux },
            stream,
          )
          .await
        }
        fun
      })
      .await?;
    let pres: PrepareRes = fetch_protocol(*capabilities, net_buffer, sequence_id, stream).await?.0;
    let _ = builder.expand(tys_len.max(pres.columns.into()), dummy_stmt_value())?;
    let elements = builder.inserted_elements();
    for _ in 0..pres.params {
      let _: ColumnRes = fetch_protocol(*capabilities, net_buffer, sequence_id, stream).await?.0;
    }
    for idx in 0..pres.columns {
      let cres = fetch_protocol(*capabilities, net_buffer, sequence_id, stream).await?.0;
      let Some(elem) = elements.get_mut(usize::from(idx)) else {
        break;
      };
      elem.0 = MysqlColumnInfo::from_column_res(&cres);
    }
    let sm = StatementsMisc::new(pres.statement_id, pres.columns.into(), 0, tys_len);
    let idx = builder.build(stmt_cmd_id, sm)?;
    let Some(stmt_mut) = stmts.get_by_idx_mut(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    Ok((stmt_cmd_id, stmt_cmd_id_array, stmt_mut))
  }
}
