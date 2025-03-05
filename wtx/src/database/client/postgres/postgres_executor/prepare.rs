use crate::{
  database::{
    DatabaseError, RecordValues, StmtCmd,
    client::{
      postgres::{
        Postgres, PostgresError, PostgresExecutor, PostgresStatement, PostgresStatements,
        column::Column,
        executor_buffer::ExecutorBuffer,
        message::MessageTy,
        msg_field::MsgField,
        postgres_executor::commons::FetchWithStmtCommons,
        protocol::{bind, describe, execute, parse, sync},
        ty::Ty,
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  misc::{
    ArrayString, LeaseMut, Stream, SuffixWriterFbvm, U64String,
    partitioned_filled_buffer::PartitionedFilledBuffer, u64_string,
  },
};

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn write_send_await_stmt_initial<RV>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    net_buffer: &mut PartitionedFilledBuffer,
    rv: RV,
    stmt: &PostgresStatement<'_>,
    stmt_cmd_id_array: &[u8],
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
  {
    {
      let mut sw = SuffixWriterFbvm::from(net_buffer._suffix_writer());
      bind(&mut sw, "", rv, stmt, stmt_cmd_id_array)?;
      execute(&mut sw, 0, "")?;
      sync(&mut sw)?;
      fwsc.stream.write_all(sw._curr_bytes()).await?;
    }
    let msg = Self::fetch_msg_from_stream(fwsc.cs, net_buffer, fwsc.stream).await?;
    let MessageTy::BindComplete = msg.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into()));
    };
    Ok(())
  }

  #[inline]
  pub(crate) async fn write_send_await_stmt_prot<'stmts, SC>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    net_buffer: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut PostgresStatements,
  ) -> Result<(u64, U64String, PostgresStatement<'stmts>), E>
  where
    S: Stream,
    SC: StmtCmd,
  {
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    if stmts.get_by_stmt_cmd_id(stmt_cmd_id).is_some() {
      // FIXME(stable): Use `if let Some ...` with polonius
      return Ok((
        stmt_cmd_id,
        stmt_cmd_id_array,
        stmts.get_by_stmt_cmd_id(stmt_cmd_id).unwrap().into(),
      ));
    }

    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;

    {
      let mut sw = SuffixWriterFbvm::from(net_buffer._suffix_writer());
      parse(
        stmt_cmd,
        &mut sw,
        fwsc.tys.iter().copied().map(Into::into),
        stmt_cmd_id_array.as_bytes(),
      )?;
      describe(stmt_cmd_id_array.as_bytes(), &mut sw, b'S')?;
      sync(&mut sw)?;
      fwsc.stream.write_all(sw._curr_bytes()).await?;
    }

    let msg0 = Self::fetch_msg_from_stream(fwsc.cs, net_buffer, fwsc.stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg0.tag }.into()));
    };

    let mut builder = stmts
      .builder((&mut *fwsc, &mut *net_buffer), {
        async fn fun<S>(
          (local_fwsc, local_nb): &mut (
            &mut FetchWithStmtCommons<'_, S>,
            &mut PartitionedFilledBuffer,
          ),
          stmt: StatementsMisc<U64String>,
        ) -> crate::Result<()>
        where
          S: Stream,
        {
          let mut sw = SuffixWriterFbvm::from(local_nb._suffix_writer());
          sw._extend_from_slices([b"S", stmt._aux.as_bytes(), &[0]])?;
          local_fwsc.stream.write_all(sw._curr_bytes()).await?;
          Ok(())
        }
        fun
      })
      .await?;

    let msg1 = Self::fetch_msg_from_stream(fwsc.cs, net_buffer, fwsc.stream).await?;
    let MessageTy::ParameterDescription(types_len, mut pd) = msg1.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into()));
    };

    let _ = builder.expand(types_len.into(), dummy())?;

    {
      let elements = builder.inserted_elements();
      for idx in 0..types_len {
        let element_opt = elements.get_mut(usize::from(idx));
        let ([a, b, c, d, sub_data @ ..], Some(element)) = (pd, element_opt) else { break };
        element.1 = Ty::Custom(u32::from_be_bytes([*a, *b, *c, *d]));
        pd = sub_data;
      }
    }

    let msg2 = Self::fetch_msg_from_stream(fwsc.cs, net_buffer, fwsc.stream).await?;
    let columns_len = match msg2.ty {
      MessageTy::NoData => 0,
      MessageTy::RowDescription(columns_len, mut rd) => {
        if let Some(diff @ 1..=u16::MAX) = columns_len.checked_sub(types_len) {
          let _ = builder.expand(diff.into(), dummy())?;
        }
        let elements = builder.inserted_elements();
        for idx in 0..columns_len {
          let (read, msg_field) = MsgField::parse(rd)?;
          let ty = Ty::Custom(msg_field.type_oid);
          let Some(element) = elements.get_mut(usize::from(idx)) else {
            break;
          };
          element.0 = Column::new(msg_field.name.try_into()?, ty);
          if let Some(elem @ [_not_empty, ..]) = rd.get(read..) {
            rd = elem;
          } else {
            break;
          }
        }
        columns_len
      }
      _ => {
        return Err(E::from(
          PostgresError::UnexpectedDatabaseMessage { received: msg2.tag }.into(),
        ));
      }
    };

    let msg3 = Self::fetch_msg_from_stream(fwsc.cs, net_buffer, fwsc.stream).await?;
    let MessageTy::ReadyForQuery = msg3.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg3.tag }.into()));
    };

    let sm = StatementsMisc::new(stmt_cmd_id_array, columns_len.into(), types_len.into());
    let idx = builder.build(stmt_cmd_id, sm)?;
    let Some(stmt) = stmts.get_by_idx(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    Ok((stmt_cmd_id, stmt_cmd_id_array, stmt.into()))
  }
}

#[inline]
fn dummy() -> (Column, Ty) {
  (Column::new(ArrayString::new(), Ty::Any), Ty::Any)
}
