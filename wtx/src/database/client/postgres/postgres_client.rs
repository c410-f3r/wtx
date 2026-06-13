mod authentication;
mod fetch;
mod simple_query;
mod stmt;

use crate::{
  collection::{TryExtend, Vector},
  database::{
    Database, DbClient, RecordValues, StmtCmd,
    client::{
      postgres::{
        Batch, Config, Postgres, PostgresError, PostgresRecords,
        client_buffer::ClientBuffer,
        message::MessageTy,
        misc::data_row,
        protocol::{encrypted_conn, initial_conn_msg},
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
  misc::{ConnectionState, LeaseMut},
  rng::CryptoRng,
  stream::Stream,
  tls::{TlsConfig, TlsConnector},
};
use core::marker::PhantomData;

/// Executor
#[derive(Debug)]
pub struct PostgresClient<CB, E, S> {
  pub(crate) cs: ConnectionState,
  pub(crate) cb: CB,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stream: S,
}

impl<CB, E, S> PostgresClient<CB, E, S>
where
  CB: LeaseMut<ClientBuffer>,
  E: From<crate::Error>,
  S: Stream,
{
  /// Connects with an unencrypted stream.
  pub async fn connect<RNG>(
    mut cb: CB,
    config: &Config<'_>,
    rng: &mut RNG,
    mut stream: S,
    tls_config: Option<&TlsConfig<'_>>,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    cb.lease_mut().clear();
    let ClientBuffer { common, conn_params: _ } = cb.lease_mut();
    let Some(elem) = tls_config else {
      return Self::do_connect(cb, config, rng, stream, None).await;
    };
    {
      let mut sw = common.read_buffer.suffix_writer();
      encrypted_conn(&mut sw)?;
      stream.write_all(sw.curr_bytes()).await?;
    }
    let mut buf = [0];
    let _ = stream.read(&mut buf).await?;
    if buf[0] != b'S' {
      return Err(PostgresError::ServerDoesNotSupportEncryption.into());
    }
    let tls_stream = TlsConnector::from_stream(stream)
      .connect(&mut common.read_buffer, None, rng, elem, &mut Vector::new())
      .await?;
    let tls_server_end_point = tls_stream.tls_server_end_point();
    //PostgresClient::do_connect(config, cb, rng, tls_stream, Some(&tls_server_end_point)).await
    todo!()
  }

  /// See [`Batch`].
  #[inline]
  pub fn batch(&mut self) -> Batch<'_, CB, E, S> {
    Batch::new(self)
  }

  /// Mutable buffer reference
  #[inline]
  pub fn eb_mut(&mut self) -> &mut ClientBuffer {
    self.cb.lease_mut()
  }

  async fn do_connect<RNG>(
    cb: CB,
    config: &Config<'_>,
    rng: &mut RNG,
    stream: S,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut this = Self { cb, cs: ConnectionState::Open, phantom: PhantomData, stream };
    this.send_initial_conn_msg(config).await?;
    this.manage_authentication(config, rng, tls_server_end_point).await?;
    this.read_after_authentication_data().await?;
    Ok(this)
  }

  async fn send_initial_conn_msg(&mut self, config: &Config<'_>) -> crate::Result<()> {
    let mut sw = self.cb.lease_mut().common.read_buffer.suffix_writer();
    initial_conn_msg(config, &mut sw)?;
    self.stream.write_all(sw.curr_bytes()).await?;
    Ok(())
  }
}

impl<CB, E, S> DbClient for PostgresClient<CB, E, S>
where
  E: From<crate::Error>,
  CB: LeaseMut<ClientBuffer>,
  S: Stream,
{
  type Database = Postgres<E>;

  fn connection_state(&self) -> ConnectionState {
    self.cs
  }

  async fn execute_many<'this, B>(
    &'this mut self,
    buffer: &mut B,
    cmd: &str,
    cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[<Self::Database as Database>::Records<'this>; 1]>,
  {
    let ClientBuffer { common, .. } = self.cb.lease_mut();
    let CommonExecutorBuffer { read_buffer, records_params, stmts, values_params, .. } = common;
    clear_cmd_buffers(read_buffer, records_params, values_params);
    Self::simple_query_execute(
      buffer,
      cmd,
      &mut self.cs,
      read_buffer,
      records_params,
      stmts,
      &mut self.stream,
      values_params,
      cb,
    )
    .await
  }

  async fn execute_stmt_many<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
    mut cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<<Self::Database as Database>::Records<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { cb: client_buffer, cs, phantom: _, stream } = self;
    let ClientBuffer { common, .. } = client_buffer.lease_mut();
    let CommonExecutorBuffer { read_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(read_buffer, records_params, values_params);
    let tuple =
      Self::write_send_await_stmt_prepare(cs, read_buffer, &rv, sc, stmts, stream).await?;
    let (_, stmt_cmd_id_array, stmt_mut) = tuple;
    Self::write_send_await_stmt_bind(cs, read_buffer, rv, &stmt_cmd_id_array, stream).await?;
    let begin = read_buffer.current_end_idx();
    let begin_data = read_buffer.current_end_idx().wrapping_add(7);
    loop {
      let msg = Self::fetch_msg_from_stream(cs, read_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        MessageTy::DataRow(values_len) => {
          data_row(
            begin,
            begin_data,
            read_buffer,
            records_params,
            stmt_mut.stmt(),
            values_len,
            values_params,
            0,
            &mut cb,
          )?;
        }
        MessageTy::ReadyForQuery => {
          break;
        }
        _ => {
          return Err(E::from(
            PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into(),
          ));
        }
      }
    }
    Ok(PostgresRecords::new(
      read_buffer.all().get(begin_data..read_buffer.current_end_idx()).unwrap_or_default(),
      records_params,
      stmt_mut.into_stmt(),
      values_params,
    ))
  }

  #[inline]
  async fn ping(&mut self) -> Result<(), E> {
    self.execute_ignored("SELECT 1").await?;
    Ok(())
  }

  async fn prepare(&mut self, cmd: &str) -> Result<u64, E> {
    let Self { cb, cs, phantom: _, stream } = self;
    let ClientBuffer { common, .. } = cb.lease_mut();
    let CommonExecutorBuffer { read_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(read_buffer, records_params, values_params);
    Ok(Self::write_send_await_stmt_prepare(cs, read_buffer, &(), cmd, stmts, stream).await?.0)
  }
}
