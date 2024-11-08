use crate::{
  database::{
    client::postgres::{
      executor::commons::FetchWithStmtCommons,
      executor_buffer::ExecutorBuffer,
      message::MessageTy,
      msg_field::MsgField,
      protocol::{bind, describe, execute, parse, sync},
      statements::{Column, Statement, StatementsMisc},
      ty::Ty,
      Executor, Postgres, PostgresError, Statements,
    },
    RecordValues, StmtCmd,
  },
  misc::{ArrayString, FilledBufferWriter, LeaseMut, PartitionedFilledBuffer, Stream},
};

impl<E, EB, S> Executor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
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
    {
      let mut fbw = FilledBufferWriter::from(&mut *nb);
      bind(&mut fbw, "", rv, stmt, stmt_id_str)?;
      execute(&mut fbw, 0, "")?;
      sync(&mut fbw)?;
      fwsc.stream.write_all(fbw._curr_bytes()).await?;
    }
    let msg = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::BindComplete = msg.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into()));
    };
    Ok(())
  }

  #[inline]
  pub(crate) async fn write_send_await_stmt_prot<'stmts, SC>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut Statements,
  ) -> Result<(u64, ArrayString<22>, Statement<'stmts>), E>
  where
    SC: StmtCmd,
  {
    let stmt_hash = sc.hash(stmts.hasher_mut());
    let stmt_id_str = Self::stmt_id_str(stmt_hash)?;
    if stmts.get_by_stmt_hash(stmt_hash).is_some() {
      // FIXME(stable): Use `if let Some ...` with polonius
      return Ok((stmt_hash, stmt_id_str, stmts.get_by_stmt_hash(stmt_hash).unwrap()));
    }

    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(PostgresError::UnknownStatementId.into()))?;

    {
      let mut fbw = FilledBufferWriter::from(&mut *nb);
      parse(stmt_cmd, &mut fbw, fwsc.tys.iter().copied().map(Into::into), &stmt_id_str)?;
      describe(&stmt_id_str, &mut fbw, b'S')?;
      sync(&mut fbw)?;
      fwsc.stream.write_all(fbw._curr_bytes()).await?;
    }

    let msg0 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg0.tag }.into()));
    };

    let msg1 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::ParameterDescription(types_len, mut pd) = msg1.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into()));
    };

    let mut builder = stmts.builder();
    let _ = builder.expand(types_len.into())?;

    {
      let elements = builder.inserted_elements();
      for idx in 0..types_len {
        let element_opt = elements.get_mut(usize::from(idx));
        let ([a, b, c, d, sub_data @ ..], Some(element)) = (pd, element_opt) else { break };
        element.1 = Ty::Custom(u32::from_be_bytes([*a, *b, *c, *d]));
        pd = sub_data;
      }
    }

    let msg2 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let columns_len = match msg2.ty {
      MessageTy::NoData => 0,
      MessageTy::RowDescription(columns_len, mut rd) => {
        if let Some(diff @ 1..=u16::MAX) = columns_len.checked_sub(types_len) {
          let _ = builder.expand(diff.into())?;
        }
        let elements = builder.inserted_elements();
        for idx in 0..columns_len {
          let (read, msg_field) = MsgField::parse(rd)?;
          let ty = Ty::Custom(msg_field.type_oid);
          let Some(element) = elements.get_mut(usize::from(idx)) else {
            break;
          };
          element.0 = Column::new(msg_field.name.try_into().map_err(Into::into)?, ty);
          if let Some(elem @ [_not_empty, ..]) = rd.get(read..) {
            rd = elem;
          } else {
            break;
          }
        }
        columns_len
      }
      _ => {
        return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg2.tag }.into()))
      }
    };

    let msg3 = Self::fetch_msg_from_stream(fwsc.cs, nb, fwsc.stream).await?;
    let MessageTy::ReadyForQuery = msg3.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg3.tag }.into()));
    };

    let sm = StatementsMisc::new(columns_len.into(), types_len.into());
    let idx = builder.build(stmt_hash, sm)?;
    let Some(stmt) = stmts.get_by_idx(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    Ok((stmt_hash, stmt_id_str, stmt))
  }

  #[inline]
  fn stmt_id_str(stmt_hash: u64) -> crate::Result<ArrayString<22>> {
    Ok(ArrayString::try_from(format_args!("s{stmt_hash}"))?)
  }
}
