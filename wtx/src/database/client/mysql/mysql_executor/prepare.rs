use crate::{
  database::{
    DatabaseError, StmtCmd,
    client::{
      mysql::{
        ExecutorBuffer, MysqlExecutor, MysqlStatement, MysqlStatementMut, MysqlStatements, Ty,
        column::Column,
        misc::{fetch_protocol, send_packet},
        mysql_protocol::{
          column_res::ColumnRes, prepare_req::PrepareReq, prepare_res::PrepareRes,
          stmt_close_req::StmtCloseReq,
        },
        ty_params::TyParams,
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  misc::{
    ArrayString, LeaseMut, Stream, U64String, Vector, net::PartitionedFilledBuffer, u64_string,
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
    encode_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut MysqlStatements,
    stream: &mut S,
    cb: impl FnOnce(&mut MysqlStatementMut<'_>) -> Result<(), E>,
  ) -> Result<(u64, U64String, MysqlStatement<'stmts>), E>
  where
    SC: StmtCmd,
  {
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    if stmts.get_by_stmt_cmd_id(stmt_cmd_id).is_some() {
      // FIXME(stable): Use `if let Some ...` with polonius
      let mut stmt = stmts.get_by_stmt_cmd_id(stmt_cmd_id).unwrap();
      cb(&mut stmt)?;
      return Ok((stmt_cmd_id, stmt_cmd_id_array, stmt.into()));
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
    let _ = builder.expand(pres.params.max(pres.columns).into(), dummy())?;
    let elements = builder.inserted_elements();
    for _ in 0..pres.params {
      let _: ColumnRes = fetch_protocol(*capabilities, net_buffer, sequence_id, stream).await?.0;
    }
    for idx in 0..pres.columns {
      let cres = fetch_protocol(*capabilities, net_buffer, sequence_id, stream).await?.0;
      let Some(elem) = elements.get_mut(usize::from(idx)) else {
        break;
      };
      elem.0 = Column::from_column_res(&cres);
    }
    let sm = StatementsMisc::new(pres.statement_id, pres.columns.into(), 0);
    let idx = builder.build(stmt_cmd_id, sm)?;
    let Some(mut stmt) = stmts.get_by_idx(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    cb(&mut stmt)?;
    Ok((stmt_cmd_id, stmt_cmd_id_array, stmt.into()))
  }
}

#[inline]
fn dummy() -> (Column, TyParams) {
  (
    Column { name: ArrayString::new(), ty_params: TyParams { flags: 0, ty: Ty::Bit } },
    TyParams { flags: 0, ty: Ty::Bit },
  )
}
