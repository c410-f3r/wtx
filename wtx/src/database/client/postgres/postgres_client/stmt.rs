use crate::{
  codec::{U64String, u64_string},
  collections::Vector,
  database::{
    DatabaseError, RecordValues, StmtCmd,
    client::{
      postgres::{
        Postgres, PostgresClient, PostgresError, PostgresStatementMut, PostgresStatements,
        message::MessageTy,
        misc::{dummy_stmt_value, row_description},
        postgres_column_info::PostgresColumnInfo,
        protocol::{bind, describe, execute, parse, sync},
        ty::Ty,
      },
      rdbms::statements_misc::StatementsMisc,
    },
  },
  misc::{ConnectionState, unlikely_elem},
  stream::{BufStreamReader, Stream, StreamWriter as _},
  tls::{TlsMode, TlsStream},
};

impl<E, S, TM> PostgresClient<E, S, TM>
where
  E: From<crate::Error>,
  S: Stream,
  TM: TlsMode,
{
  pub(crate) async fn await_stmt_bind(
    cs: &mut ConnectionState,
    read_buffer: &mut BufStreamReader,
    stream: &mut TlsStream<S, TM, true>,
  ) -> Result<(), E>
  where
    S: Stream,
  {
    let msg = Self::fetch_msg(cs, read_buffer, stream).await?;
    let MessageTy::BindComplete = msg.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into()));
    };
    Ok(())
  }

  #[expect(clippy::wildcard_enum_match_arm, reason = "too many variants")]
  pub(crate) async fn await_stmt_prepare<'stmts, const HAS_SYNC: bool>(
    cs: &mut ConnectionState,
    read_buffer: &mut BufStreamReader,
    stmt_cmd_id: u64,
    stmt_cmd_id_array: U64String,
    stmts: &'stmts mut PostgresStatements,
    stream: &mut TlsStream<S, TM, true>,
  ) -> Result<PostgresStatementMut<'stmts>, E>
  where
    S: Stream,
  {
    let msg0 = Self::fetch_msg(cs, read_buffer, stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(E::from(PostgresError::UnexpectedDatabaseMessage { received: msg0.tag }.into()));
    };

    let mut builder = stmts
      .builder((&mut *read_buffer, &mut *stream), {
        async fn fun<S>(
          _: &mut (&mut BufStreamReader, &mut S),
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

    let pd_begin = read_buffer.current_end_idx().wrapping_add(6);
    let types_len = {
      let msg1 = Self::fetch_msg(cs, read_buffer, stream).await?;
      let MessageTy::ParameterDescription(types_len) = msg1.ty else {
        return Err(E::from(
          PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into(),
        ));
      };
      types_len
    };
    let pd_end = read_buffer.current_end_idx();

    let msg2 = Self::fetch_msg(cs, read_buffer, stream).await?;
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
    let bytes = read_buffer.filled().get(pd_begin..pd_end);
    parameter_description(bytes, pd_data).ok_or(crate::Error::ProgrammingError)?;

    if HAS_SYNC {
      let msg3 = Self::fetch_msg(cs, read_buffer, stream).await?;
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
    read_buffer: &mut BufStreamReader,
    rv: RV,
    stmt_cmd_id_array: &U64String,
    stream: &mut TlsStream<S, TM, true>,
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
  {
    {
      let mut sw = read_buffer.suffix_pusher();
      Self::write_stmt_bind::<_, true>(rv, stmt_cmd_id_array, sw.inner_mut())?;
      stream.write_all(sw.curr()).await?;
    }
    Self::await_stmt_bind(cs, read_buffer, stream).await
  }

  pub(crate) async fn write_send_await_stmt_prepare<'stmts, RV, SC>(
    cs: &mut ConnectionState,
    read_buffer: &mut BufStreamReader,
    rv: &RV,
    sc: SC,
    stmts: &'stmts mut PostgresStatements,
    stream: &mut TlsStream<S, TM, true>,
  ) -> Result<(u64, U64String, PostgresStatementMut<'stmts>), E>
  where
    RV: RecordValues<Postgres<E>>,
    S: Stream,
    SC: StmtCmd,
    TM: TlsMode,
  {
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    if stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).is_some() {
      // FIXME(STABLE): Use `if let Some ...` with polonius
      #[expect(clippy::unwrap_used, reason = "borrow-checker")]
      return Ok((
        stmt_cmd_id,
        stmt_cmd_id_array,
        stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).unwrap(),
      ));
    }
    let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;
    {
      let mut sw = read_buffer.suffix_pusher();
      Self::write_stmt_prepare::<_, true>(sw.inner_mut(), rv, stmt_cmd, &stmt_cmd_id_array)?;
      stream.write_all(sw.curr()).await?;
    }
    let stmt_mut = Self::await_stmt_prepare::<true>(
      cs,
      read_buffer,
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
    sw: &mut Vector<u8>,
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

  pub(crate) fn write_stmt_prepare<RV, const SYNC: bool>(
    buffer: &mut Vector<u8>,
    rv: &RV,
    stmt_cmd: &str,
    stmt_cmd_id_array: &U64String,
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
    S: Stream,
  {
    parse(buffer, rv, stmt_cmd, stmt_cmd_id_array)?;
    describe(buffer, stmt_cmd_id_array.as_bytes(), b'S')?;
    if SYNC {
      sync(buffer)?;
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
    let [b0, b1, b2, b3, rest @ ..] = local_bytes else {
      return unlikely_elem(None);
    };
    element.1 = Ty::from_arbitrary_u32(u32::from_be_bytes([*b0, *b1, *b2, *b3]));
    *local_bytes = rest;
  }
  Some(())
}
