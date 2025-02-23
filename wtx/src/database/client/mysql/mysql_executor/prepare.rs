use crate::{
  database::{
    DatabaseError, StmtCmd,
    client::{
      mysql::{
        ExecutorBuffer, MysqlExecutor, MysqlStatement, MysqlStatements, Ty,
        column::Column,
        misc::{fetch_protocol, send_packet},
        mysql_protocol::{
          column_res::ColumnRes, prepare_req::PrepareReq, prepare_res::PrepareRes,
          stmt_close_req::StmtCloseReq,
        },
        ty_params::TyParams,
      },
      rdbms::{U64Array, statements_misc::StatementsMisc, u64_array},
    },
  },
  misc::{
    ArrayString, LeaseMut, Stream, Vector, partitioned_filled_buffer::PartitionedFilledBuffer,
  },
};

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn write_send_await_stmt_prot<'stmts, SC>(
    (capabilities, sequence_id): (&mut u64, &mut u8),
    enc_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut MysqlStatements,
    stream: &mut S,
  ) -> Result<(u64, U64Array, MysqlStatement<'stmts>), E>
  where
    SC: StmtCmd,
  {
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_array(stmt_cmd_id);
    if stmts.get_by_stmt_cmd_id(stmt_cmd_id).is_some() {
      // FIXME(stable): Use `if let Some ...` with polonius
      return Ok((stmt_cmd_id, stmt_cmd_id_array, stmts.get_by_stmt_cmd_id(stmt_cmd_id).unwrap()));
    }

    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;

    send_packet(
      (capabilities, sequence_id),
      enc_buffer,
      PrepareReq { query: stmt_cmd.as_bytes() },
      stream,
    )
    .await?;
    let prepare_res: PrepareRes = fetch_protocol(net_buffer, sequence_id, stream).await?;
    let mut builder = stmts.builder();
    let _ = builder.expand(prepare_res.params.max(prepare_res.columns).into(), dummy())?;
    let elements = builder.inserted_elements();
    for idx in 0..prepare_res.params {
      let res: ColumnRes = fetch_protocol(net_buffer, sequence_id, stream).await?;
      let Some(elem0) = elements.get_mut(usize::from(idx)) else {
        break;
      };
      elem0.1 = Column::from_column_res(&res).ty_params;
    }
    for idx in 0..prepare_res.columns {
      let res: ColumnRes = fetch_protocol(net_buffer, sequence_id, stream).await?;
      let Some(elem) = elements.get_mut(usize::from(idx)) else {
        break;
      };
      elem.0 = Column::from_column_res(&res);
    }
    let sm = StatementsMisc::new(
      prepare_res.statement_id,
      prepare_res.columns.into(),
      prepare_res.params.into(),
    );
    let idx = builder.build(stmt_cmd_id, sm)?;
    enc_buffer.clear();
    send_packet(
      (capabilities, sequence_id),
      enc_buffer,
      StmtCloseReq { statement: prepare_res.statement_id },
      stream,
    )
    .await?;
    let Some(stmt) = stmts.get_by_idx(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    Ok((stmt_cmd_id, stmt_cmd_id_array, stmt))
  }
}

#[inline]
fn dummy() -> (Column, TyParams) {
  (
    Column { name: ArrayString::new(), ty_params: TyParams { flags: 0, ty: Ty::Bit } },
    TyParams { flags: 0, ty: Ty::Bit },
  )
}
