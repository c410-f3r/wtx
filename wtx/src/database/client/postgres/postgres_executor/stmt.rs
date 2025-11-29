use crate::{
  database::{
    DatabaseError, RecordValues, StmtCmd,
    client::{
      postgres::{
        Postgres, PostgresError, PostgresExecutor, PostgresStatementMut, PostgresStatements,
        executor_buffer::ExecutorBuffer,
        message::MessageTy,
        misc::{dummy_stmt_value, row_description},
        postgres_column_info::PostgresColumnInfo,
        protocol::{bind, describe, execute, parse, sync},
        ty::Ty,
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  de::{U64String, u64_string},
  misc::{
    ConnectionState, FilledBufferVectorMut, LeaseMut, SuffixWriter, SuffixWriterFbvm,
    net::PartitionedFilledBuffer, unlikely_elem,
  },
  stream::Stream,
};

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn await_stmt_bind(
    cs: &mut ConnectionState,
    net_buffer: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> Result<(), E>
  where
    S: Stream,
  {
    let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
    let MessageTy::BindComplete = msg.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into()));
    };
    Ok(())
  }

  pub(crate) async fn await_stmt_prepare<'stmts, const HAS_SYNC: bool>(
    cs: &mut ConnectionState,
    net_buffer: &mut PartitionedFilledBuffer,
    stmt_cmd_id: u64,
    stmt_cmd_id_array: U64String,
    stmts: &'stmts mut PostgresStatements,
    stream: &mut S,
  ) -> Result<PostgresStatementMut<'stmts>, E>
  where
    S: Stream,
  {
    let msg0 = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg0.tag }.into()));
    };

    let mut builder = stmts
      .builder((&mut *net_buffer, &mut *stream), {
        async fn fun<S>(
          _: &mut (&mut PartitionedFilledBuffer, &mut S),
          _: StatementsMisc<U64String>,
        ) -> crate::Result<()>
        where
          S: Stream,
        {
          Ok(())
        }
        fun
      })
      .await?;

    let pd_begin = net_buffer.current_end_idx().wrapping_add(6);
    let types_len = {
      let msg1 = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      let MessageTy::ParameterDescription(types_len) = msg1.ty else {
        return Err(E::from(
          PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into(),
        ));
      };
      types_len
    };
    let pd_end = net_buffer.current_end_idx();

    let msg2 = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
    let (columns_len, pd_data) = match msg2.ty {
      MessageTy::NoData => (0, builder.expand(types_len.into(), dummy_stmt_value())?),
      MessageTy::RowDescription(columns_len, mut rd) => {
        let data = builder.expand(types_len.max(columns_len).into(), dummy_stmt_value())?;
        row_description(columns_len, &mut rd, |idx, pci| {
          if let Some(element) = data.get_mut(usize::from(idx)) {
            element.0 = pci;
          }
          Ok(())
        })?;
        (columns_len, data.get_mut(..types_len.into()).unwrap_or_default())
      }
      _ => {
        return Err(E::from(
          PostgresError::UnexpectedDatabaseMessage { received: msg2.tag }.into(),
        ));
      }
    };
    let bytes = net_buffer.all().get(pd_begin..pd_end);
    parameter_description(bytes, pd_data).ok_or(crate::Error::ProgrammingError)?;

    if HAS_SYNC {
      let msg3 = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      let MessageTy::ReadyForQuery = msg3.ty else {
        return Err(E::from(
          PostgresError::UnexpectedDatabaseMessage { received: msg3.tag }.into(),
        ));
      };
    }

    let sm = StatementsMisc::new(stmt_cmd_id_array, columns_len.into(), 0, types_len.into());
    let idx = builder.build(stmt_cmd_id, sm)?;
    let Some(stmt_mut) = stmts.get_by_idx_mut(idx) else {
      return Err(crate::Error::ProgrammingError.into());
    };
    Ok(stmt_mut)
  }

  pub(crate) async fn write_send_await_stmt_bind<RV>(
    cs: &mut ConnectionState,
    net_buffer: &mut PartitionedFilledBuffer,
    rv: RV,
    stmt_cmd_id_array: &U64String,
    stream: &mut S,
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
  {
    {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      Self::write_stmt_bind::<_, true>(rv, stmt_cmd_id_array, &mut sw)?;
      stream.write_all(sw.curr_bytes()).await?;
    }
    Self::await_stmt_bind(cs, net_buffer, stream).await
  }

  pub(crate) async fn write_send_await_stmt_prepare<'stmts, SC>(
    cs: &mut ConnectionState,
    net_buffer: &mut PartitionedFilledBuffer,
    sc: SC,
    stmts: &'stmts mut PostgresStatements,
    stream: &mut S,
    tys: &[Ty],
  ) -> Result<(u64, U64String, PostgresStatementMut<'stmts>), E>
  where
    S: Stream,
    SC: StmtCmd,
  {
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    if stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).is_some() {
      // FIXME(STABLE): Use `if let Some ...` with polonius
      return Ok((
        stmt_cmd_id,
        stmt_cmd_id_array,
        stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).unwrap(),
      ));
    }
    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;
    {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      Self::write_stmt_prepare::<true>(stmt_cmd, &stmt_cmd_id_array, &mut sw, tys)?;
      stream.write_all(sw.curr_bytes()).await?;
    }
    let stmt_mut = Self::await_stmt_prepare::<true>(
      cs,
      net_buffer,
      stmt_cmd_id,
      stmt_cmd_id_array,
      stmts,
      stream,
    )
    .await?;
    Ok((stmt_cmd_id, stmt_cmd_id_array, stmt_mut))
  }

  pub(crate) fn write_stmt_bind<RV, const SYNC: bool>(
    rv: RV,
    stmt_cmd_id_array: &U64String,
    sw: &mut SuffixWriter<FilledBufferVectorMut<'_>>,
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
  {
    bind(sw, "", rv, stmt_cmd_id_array)?;
    execute(sw, 0, "")?;
    if SYNC {
      sync(sw)?;
    }
    Ok(())
  }

  pub(crate) fn write_stmt_prepare<const SYNC: bool>(
    stmt_cmd: &str,
    stmt_cmd_id_array: &U64String,
    sw: &mut SuffixWriter<FilledBufferVectorMut<'_>>,
    tys: &[Ty],
  ) -> Result<(), E>
  where
    S: Stream,
  {
    parse(stmt_cmd, sw, tys.iter().copied().map(Into::into), stmt_cmd_id_array)?;
    describe(stmt_cmd_id_array.as_bytes(), sw, b'S')?;
    if SYNC {
      sync(sw)?;
    }
    Ok(())
  }
}

fn parameter_description(
  bytes: Option<&[u8]>,
  elements: &mut [(PostgresColumnInfo, Ty)],
) -> Option<()> {
  let local_bytes = &mut bytes?;
  for element in elements {
    let [a, b, c, d, rest @ ..] = local_bytes else {
      return unlikely_elem(None);
    };
    element.1 = Ty::Custom(u32::from_be_bytes([*a, *b, *c, *d]));
    *local_bytes = rest;
  }
  Some(())
}
