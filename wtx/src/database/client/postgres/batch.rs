use core::mem::ManuallyDrop;

use crate::{
  codec::u64_string,
  collections::{ArrayVector, ArrayVectorU8, TryExtend},
  database::{
    Database, DatabaseError, RecordValues, StmtCmd,
    client::{
      postgres::{
        Postgres, PostgresClient, PostgresError, PostgresRecord,
        client_buffer::ClientBuffer,
        message::MessageTy,
        misc::{data_row, extend_records, row_description},
        protocol::sync,
      },
      rdbms::{clear_query_buffers, common_client_buffer::CommonClientBuffer},
    },
  },
  misc::{Either, Usize},
  stream::{Stream, StreamWriter as _},
  tls::TlsMode,
};

const MAX_STMTS: usize = 4;

/// Sends multiple statements at once and awaits them when flushed.
#[derive(Debug)]
pub struct Batch<'exec, E, S, TM> {
  client: &'exec mut PostgresClient<E, S, TM>,
  initial_len: usize,
  stmt_cmd_ids: ArrayVectorU8<(u64, bool), MAX_STMTS>,
}

impl<'exec, E, S, TM> Batch<'exec, E, S, TM>
where
  E: From<crate::Error>,
  S: Stream,
  TM: TlsMode,
{
  pub(crate) fn new(client: &'exec mut PostgresClient<E, S, TM>) -> Self {
    let PostgresClient { cb, .. } = client;
    let ClientBuffer { common, .. } = cb;
    let CommonClientBuffer { read_buffer, records_params, values_params, .. } = common;
    clear_query_buffers(records_params, values_params);
    let initial_len = read_buffer.filled().len();
    *read_buffer.forbid_clear_mut() = true;
    Self { client, initial_len, stmt_cmd_ids: ArrayVector::new() }
  }

  /// Combines received results based on previous [`Self::stmt`] calls.
  #[expect(clippy::wildcard_enum_match_arm, reason = "too many variants")]
  #[inline]
  pub async fn flush<'this, B>(
    &'this mut self,
    buffer: &mut B,
    mut cb: impl FnMut(PostgresRecord<'_, E>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[<Postgres<E> as Database>::Records<'this>; 1]>,
  {
    let PostgresClient { cb: client_buffer, cs, phantom: _, stream } = self.client;
    let ClientBuffer { common, .. } = client_buffer;
    let CommonClientBuffer { read_buffer, records_params, stmts, values_params } = common;
    {
      let mut sw = ManuallyDrop::new(read_buffer.suffix_pusher());
      sync(&mut sw)?;
      stream.write_all(sw.inner_mut().get(self.initial_len..).unwrap_or_default()).await?;
      sw.inner_mut().truncate(self.initial_len);
    }

    let begin_data = read_buffer.current_end_idx().wrapping_add(7);
    let mut stmt_cmd_ids_iter = self.stmt_cmd_ids.iter().copied();
    let mut values_params_offset = 0;
    'stmts: loop {
      let Some((stmt_cmd_id, is_already_known)) = stmt_cmd_ids_iter.next() else {
        let msg = PostgresClient::<E, S, TM>::fetch_msg(cs, read_buffer, stream).await?;
        if let MessageTy::ReadyForQuery = msg.ty {
          break 'stmts;
        }
        return Err(crate::Error::ProgrammingError.into());
      };
      let stmt_mut = if is_already_known {
        stmts
          .get_by_stmt_cmd_id_mut(stmt_cmd_id)
          .ok_or_else(|| E::from(crate::Error::ProgrammingError))?
      } else {
        PostgresClient::<E, S, TM>::await_stmt_prepare::<false>(
          cs,
          read_buffer,
          stmt_cmd_id,
          u64_string(stmt_cmd_id),
          stmts,
          stream,
        )
        .await?
      };
      PostgresClient::<E, S, TM>::await_stmt_bind(cs, read_buffer, stream).await?;

      'rows: loop {
        let msg = PostgresClient::<E, S, TM>::fetch_msg(cs, read_buffer, stream).await?;
        match msg.ty {
          MessageTy::CommandComplete(rows_len) => {
            if !B::IS_UNIT {
              *stmt_mut.rows_len = *Usize::from(rows_len);
              values_params_offset = values_params.len();
            }
            break 'rows;
          }
          MessageTy::DataRow(values_len) => {
            if !B::IS_UNIT {
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
          MessageTy::ReadyForQuery => break 'stmts,
          MessageTy::RowDescription(columns_len, mut rd) => {
            if !B::IS_UNIT {
              row_description(columns_len, &mut rd, |_, _| Ok(()))?;
            }
          }
          _ => {
            let received = msg.tag;
            return Err(
              crate::Error::from(PostgresError::UnexpectedDatabaseMessage { received }).into(),
            );
          }
        }
      }
    }
    extend_records(
      begin_data,
      buffer,
      read_buffer,
      records_params,
      stmts,
      self.stmt_cmd_ids.iter().map(|el| Either::Right(el.0)),
      values_params,
    )?;
    Ok(())
  }

  /// Pushes a single statement for a later submission.
  #[inline]
  pub fn stmt<SC, RV>(&mut self, sc: SC, rv: RV) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
    SC: StmtCmd,
  {
    let PostgresClient { cb, .. } = self.client;
    let ClientBuffer { common, .. } = cb;
    let CommonClientBuffer { read_buffer, stmts, .. } = common;
    let stmt_cmd_id = sc.hash(stmts.hasher_mut());
    let stmt_cmd_id_array = u64_string(stmt_cmd_id);
    let is_already_known_externally = stmts.get_by_stmt_cmd_id_mut(stmt_cmd_id).is_some();
    let is_already_known_internally = self.stmt_cmd_ids.iter().any(|&(id, _)| id == stmt_cmd_id);
    let is_already_known = is_already_known_externally || is_already_known_internally;
    let mut sw = ManuallyDrop::new(read_buffer.suffix_pusher());
    if !is_already_known {
      let stmt_cmd = sc.cmd().ok_or_else(|| E::from(DatabaseError::UnknownStatementId.into()))?;
      PostgresClient::<E, S, TM>::write_stmt_prepare::<_, false>(
        &rv,
        stmt_cmd,
        &stmt_cmd_id_array,
        &mut sw,
      )?;
    }
    PostgresClient::<E, S, TM>::write_stmt_bind::<_, false>(rv, &stmt_cmd_id_array, &mut sw)?;
    self.stmt_cmd_ids.push((stmt_cmd_id, is_already_known))?;
    Ok(())
  }
}

impl<E, S, TM> Drop for Batch<'_, E, S, TM> {
  #[inline]
  fn drop(&mut self) {
    *self.client.cb.common.read_buffer.forbid_clear_mut() = false;
    ManuallyDrop::new(self.client.cb.common.read_buffer.suffix_pusher())
      .inner_mut()
      .truncate(self.initial_len);
  }
}
