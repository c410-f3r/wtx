use crate::{
  database::{
    client::postgres::{
      bind, describe, execute,
      executor::commons::FetchWithStmtCommons,
      executor_buffer::ExecutorBuffer,
      parse,
      statements::{Column, PushRslt, Statement},
      sync,
      ty::Ty,
      Executor, MessageTy, MsgField, Postgres, PostgresError, Statements,
    },
    RecordValues, StmtCmd,
  },
  misc::{
    ArrayString, FilledBufferWriter, LeaseMut, PartitionedFilledBuffer, Stream, _unreachable,
  },
};
use core::ops::Range;

impl<E, EB, S> Executor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn write_send_await_stmt_initial<RV>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    rv: RV,
    stmt: &Statement<'_>,
    stmt_id_str: &str,
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
  {
    let mut fbw = FilledBufferWriter::from(&mut *nb);
    bind(&mut fbw, "", rv, stmt, stmt_id_str)?;
    execute(&mut fbw, 0, "")?;
    sync(&mut fbw)?;
    fwsc.stream.write_all(fbw._curr_bytes()).await?;
    let msg = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::BindComplete = msg.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into()));
    };
    Ok(())
  }

  pub(crate) async fn write_send_await_stmt_prot<'stmts, SC>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut Statements,
    _vb: &mut [(bool, Range<usize>)],
  ) -> Result<(u64, ArrayString<22>, Statement<'stmts>), E>
  where
    SC: StmtCmd,
  {
    let stmt_hash = sc.hash(stmts.hasher_mut());
    let (stmt_id_str, mut builder) = match stmts.push(stmt_hash) {
      PushRslt::Builder(builder) => (Self::stmt_id_str(stmt_hash)?, builder),
      PushRslt::Stmt(stmt) => return Ok((stmt_hash, Self::stmt_id_str(stmt_hash)?, stmt)),
    };

    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(PostgresError::UnknownStatementId.into()))?;

    let mut fbw = FilledBufferWriter::from(&mut *nb);
    parse(stmt_cmd, &mut fbw, fwsc.tys.iter().copied().map(Into::into), &stmt_id_str)?;
    describe(&stmt_id_str, &mut fbw, b'S')?;
    sync(&mut fbw)?;
    fwsc.stream.write_all(fbw._curr_bytes()).await?;

    let msg0 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg0.tag }.into()));
    };

    let msg1 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::ParameterDescription(mut pd) = msg1.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into()));
    };
    while let [a, b, c, d, sub_data @ ..] = pd {
      let id = u32::from_be_bytes([*a, *b, *c, *d]);
      builder.push_param(Ty::Custom(id));
      pd = sub_data;
    }

    let msg2 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    match msg2.ty {
      MessageTy::NoData => {}
      MessageTy::RowDescription(mut rd) => loop {
        let (read, msg_field) = MsgField::parse(rd)?;
        let ty = Ty::Custom(msg_field.type_oid);
        builder.push_column(Column { name: msg_field.name.try_into().map_err(Into::into)?, ty });
        if let Some(elem @ [_not_empty, ..]) = rd.get(read..) {
          rd = elem;
        } else {
          break;
        }
      },
      _ => {
        return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg2.tag }.into()))
      }
    }

    let msg3 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::ReadyForQuery = msg3.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg3.tag }.into()));
    };

    if let Some(stmt) = builder.finish().get_by_stmt_hash(stmt_hash) {
      Ok((stmt_hash, stmt_id_str, stmt))
    } else {
      _unreachable()
    }
  }

  fn stmt_id_str(stmt_hash: u64) -> crate::Result<ArrayString<22>> {
    Ok(ArrayString::try_from(format_args!("s{stmt_hash}"))?)
  }
}
