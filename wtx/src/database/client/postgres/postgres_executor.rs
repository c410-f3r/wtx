mod authentication;
mod commons;
mod fetch;
mod prepare;
mod simple_query;

use crate::{
  collection::IndexedStorageMut,
  database::{
    Database, Executor, RecordValues, StmtCmd,
    client::{
      postgres::{
        Config, Postgres, PostgresError, PostgresRecord, PostgresRecords,
        executor_buffer::ExecutorBuffer,
        message::MessageTy,
        postgres_executor::commons::FetchWithStmtCommons,
        protocol::{encrypted_conn, initial_conn_msg},
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
  de::DEController,
  misc::{ConnectionState, Lease, LeaseMut},
  rng::CryptoRng,
  stream::{Stream, StreamWithTls},
};
use core::marker::PhantomData;

/// Executor
#[derive(Debug)]
pub struct PostgresExecutor<E, EB, S> {
  pub(crate) cs: ConnectionState,
  pub(crate) eb: EB,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stream: S,
}

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  /// Connects with an unencrypted stream.
  pub async fn connect<RNG>(
    config: &Config<'_>,
    mut eb: EB,
    rng: &mut RNG,
    stream: S,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    eb.lease_mut().clear();
    Self::do_connect(config, eb, rng, stream, None).await
  }

  /// Initially connects with an unencrypted stream that should be later upgraded to an encrypted
  /// stream.
  pub async fn connect_encrypted<F, IS, RNG>(
    config: &Config<'_>,
    mut eb: EB,
    rng: &mut RNG,
    mut stream: IS,
    cb: impl FnOnce(IS) -> F,
  ) -> crate::Result<Self>
  where
    F: Future<Output = crate::Result<S>>,
    IS: Stream,
    RNG: CryptoRng,
    S: StreamWithTls,
  {
    eb.lease_mut().clear();
    {
      let mut sw = eb.lease_mut().common.net_buffer.suffix_writer();
      encrypted_conn(&mut sw)?;
      stream.write_all(sw.curr_bytes()).await?;
    }
    let mut buf = [0];
    let _ = stream.read(&mut buf).await?;
    if buf[0] != b'S' {
      return Err(PostgresError::ServerDoesNotSupportEncryption.into());
    }
    let enc_stream = cb(stream).await?;
    let tls_server_end_point = enc_stream.tls_server_end_point()?;
    Self::do_connect(config, eb, rng, enc_stream, tls_server_end_point.as_ref().map(Lease::lease))
      .await
  }

  /// Mutable buffer reference
  pub fn eb_mut(&mut self) -> &mut ExecutorBuffer {
    self.eb.lease_mut()
  }

  async fn do_connect<RNG>(
    config: &Config<'_>,
    eb: EB,
    rng: &mut RNG,
    stream: S,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut this = Self { eb, cs: ConnectionState::Open, phantom: PhantomData, stream };
    this.send_initial_conn_msg(config).await?;
    this.manage_authentication(config, rng, tls_server_end_point).await?;
    this.read_after_authentication_data().await?;
    Ok(this)
  }

  async fn send_initial_conn_msg(&mut self, config: &Config<'_>) -> crate::Result<()> {
    let mut sw = self.eb.lease_mut().common.net_buffer.suffix_writer();
    initial_conn_msg(config, &mut sw)?;
    self.stream.write_all(sw.curr_bytes()).await?;
    Ok(())
  }
}

impl<E, EB, S> Executor for PostgresExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  type Database = Postgres<E>;

  fn connection_state(&self) -> ConnectionState {
    self.cs
  }

  async fn execute(
    &mut self,
    cmd: &str,
    cb: impl FnMut(u64) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> Result<(), <Self::Database as DEController>::Error> {
    let ExecutorBuffer { common, .. } = self.eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, values_params, .. } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Self::simple_query_execute(cmd, &mut self.cs, net_buffer, &mut self.stream, cb).await
  }

  async fn execute_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> Result<u64, <Self::Database as DEController>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let mut rows = 0;
    let mut fwsc = FetchWithStmtCommons { cs, stream, tys: &[] };
    let (_, stmt_cmd_id, stmt) =
      Self::write_send_await_stmt_prot(&mut fwsc, net_buffer, sc, stmts).await?;
    Self::write_send_await_stmt_initial(&mut fwsc, net_buffer, rv, &stmt, stmt_cmd_id.as_bytes())
      .await?;
    loop {
      let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(local_rows) => {
          rows = local_rows;
        }
        MessageTy::ReadyForQuery => break,
        MessageTy::DataRow(_) | MessageTy::EmptyQueryResponse => {}
        _ => {
          return Err(<_>::from(
            PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into(),
          ));
        }
      }
    }
    Ok(rows)
  }

  async fn fetch_many_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
    mut cb: impl FnMut(&<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<<Self::Database as Database>::Records<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let mut fwsc = FetchWithStmtCommons { cs, stream, tys: &[] };
    let (_, stmt_cmd_id_array, stmt) =
      Self::write_send_await_stmt_prot(&mut fwsc, net_buffer, sc, stmts).await?;
    Self::write_send_await_stmt_initial(
      &mut fwsc,
      net_buffer,
      rv,
      &stmt,
      stmt_cmd_id_array.as_bytes(),
    )
    .await?;
    let begin = net_buffer.current_end_idx();
    let begin_data = net_buffer.current_end_idx().wrapping_add(7);
    loop {
      let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        MessageTy::DataRow(values_len) => {
          let net_buffer_range = begin_data..net_buffer.current_end_idx();
          let mut bytes = net_buffer.all().get(net_buffer_range).unwrap_or_default();
          let record_range_begin = net_buffer.antecedent_end_idx().wrapping_sub(begin);
          let record_range_end = net_buffer.current_end_idx().wrapping_sub(begin_data);
          bytes = bytes.get(record_range_begin..record_range_end).unwrap_or_default();
          let values_params_begin = values_params.len();
          cb(&PostgresRecord::parse(bytes, stmt.clone(), values_len, values_params)?)?;
          records_params.push((
            record_range_begin..record_range_end,
            values_params_begin..values_params.len(),
          ))?;
        }
        MessageTy::ReadyForQuery => {
          break;
        }
        _ => {
          return Err(<_>::from(
            PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into(),
          ));
        }
      }
    }
    Ok(PostgresRecords::new(
      net_buffer.all().get(begin_data..net_buffer.current_end_idx()).unwrap_or_default(),
      records_params,
      stmt,
      values_params,
    ))
  }

  async fn fetch_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params, .. } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let mut fwsc = FetchWithStmtCommons { cs, stream, tys: &[] };
    let (_, stmt_cmd_id, stmt) =
      Self::write_send_await_stmt_prot(&mut fwsc, net_buffer, sc, stmts).await?;
    Self::write_send_await_fetch_with_stmt_wo_prot(
      &mut fwsc,
      net_buffer,
      rv,
      stmt,
      stmt_cmd_id.as_bytes(),
      values_params,
    )
    .await
  }

  async fn prepare(&mut self, cmd: &str) -> Result<u64, E> {
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let mut fwsc = FetchWithStmtCommons { cs, stream, tys: &[] };
    Ok(Self::write_send_await_stmt_prot(&mut fwsc, net_buffer, cmd, stmts).await?.0)
  }
}
