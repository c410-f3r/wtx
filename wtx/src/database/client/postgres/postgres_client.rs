mod authentication;
mod fetch;
mod simple_query;
mod stmt;

use crate::{
  collections::TryExtend,
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
      rdbms::{clear_query_buffers, common_client_buffer::CommonClientBuffer},
    },
  },
  misc::{ConnectionState, Lease, SingleTypeStorage},
  rng::CryptoRng,
  stream::{Stream, StreamWriter as _},
  tls::{TlsConfig, TlsConnector, TlsMode, TlsServerEndPoint, TlsStream},
};
use core::marker::PhantomData;

/// Executor
#[derive(Debug)]
pub struct PostgresClient<E, S, TM> {
  pub(crate) cs: ConnectionState,
  pub(crate) cb: ClientBuffer,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stream: TlsStream<S, TM, true>,
}

impl<E, S, TM> PostgresClient<E, S, TM>
where
  E: From<crate::Error>,
  S: Stream,
  TM: TlsMode,
{
  /// Connects with an unencrypted stream.
  #[inline]
  pub async fn connect<RNG, TC>(
    mut client_buffer: ClientBuffer,
    config: &Config<'_>,
    mut tls_connector: TlsConnector<RNG, S, TC>,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
    TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
  {
    client_buffer.clear();
    let ClientBuffer { common, conn_params: _ } = &mut client_buffer;
    if TM::TY.is_plain_text() {
      let mut rslt = tls_connector.connect().await?.rslt()?;
      return Self::do_connect(
        client_buffer,
        config,
        &mut rslt.rng,
        rslt.stream,
        rslt.server_end_point,
      )
      .await;
    }
    {
      let mut sw = common.read_buffer.suffix_pusher();
      encrypted_conn(sw.inner_mut())?;
      tls_connector.stream_mut().write_all(sw.curr()).await?;
    }
    let mut buf = [0];
    let _read = tls_connector.stream_mut().read(buf.as_mut_slice().into()).await?.rslt()?;
    if buf != *b"S" {
      return Err(PostgresError::ServerDoesNotSupportEncryption.into());
    }
    let mut rslt = tls_connector.connect().await?.rslt()?;
    return Self::do_connect(
      client_buffer,
      config,
      &mut rslt.rng,
      rslt.stream,
      rslt.server_end_point,
    )
    .await;
  }

  /// See [`Batch`].
  #[inline]
  pub fn batch(&mut self) -> Batch<'_, E, S, TM> {
    Batch::new(self)
  }

  /// Mutable buffer reference
  #[inline]
  pub fn cb_mut(&mut self) -> &mut ClientBuffer {
    &mut self.cb
  }

  async fn do_connect<RNG>(
    cb: ClientBuffer,
    config: &Config<'_>,
    rng: &mut RNG,
    stream: TlsStream<S, TM, true>,
    tls_server_end_point: TlsServerEndPoint,
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
    let mut sw = self.cb.common.read_buffer.suffix_pusher();
    initial_conn_msg(config, sw.inner_mut())?;
    self.stream.write_all(sw.curr()).await?;
    Ok(())
  }
}

impl<E, S, TM> DbClient for PostgresClient<E, S, TM>
where
  E: From<crate::Error>,
  S: Stream,
  TM: TlsMode,
{
  type Database = Postgres<E>;

  #[inline]
  fn connection_state(&self) -> ConnectionState {
    self.cs
  }

  #[inline]
  async fn execute_many<'this, B>(
    &'this mut self,
    buffer: &mut B,
    cmd: &str,
    cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[<Self::Database as Database>::Records<'this>; 1]>,
  {
    let ClientBuffer { common, .. } = &mut self.cb;
    let CommonClientBuffer { read_buffer, records_params, stmts, values_params } = common;
    clear_query_buffers(records_params, values_params);
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

  #[expect(clippy::wildcard_enum_match_arm, reason = "too many variants")]
  #[inline]
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
    let ClientBuffer { common, .. } = client_buffer;
    let CommonClientBuffer { read_buffer, records_params, stmts, values_params } = common;
    clear_query_buffers(records_params, values_params);
    let tuple =
      Self::write_send_await_stmt_prepare(cs, read_buffer, &rv, sc, stmts, stream).await?;
    let (_, stmt_cmd_id_array, stmt_mut) = tuple;
    Self::write_send_await_stmt_bind(cs, read_buffer, rv, &stmt_cmd_id_array, stream).await?;
    let begin_data = read_buffer.current_end_idx().wrapping_add(7);
    *read_buffer.forbid_clear_mut() = true;
    loop {
      let msg = Self::fetch_msg(cs, read_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        MessageTy::DataRow(values_len) => {
          data_row(
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
    *read_buffer.forbid_clear_mut() = false;
    Ok(PostgresRecords::new(
      read_buffer.filled().get(begin_data..read_buffer.current_end_idx()).unwrap_or_default(),
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

  #[inline]
  async fn prepare(&mut self, cmd: &str) -> Result<u64, E> {
    let Self { cb, cs, phantom: _, stream } = self;
    let ClientBuffer { common, .. } = cb;
    let CommonClientBuffer { read_buffer, records_params, stmts, values_params } = common;
    clear_query_buffers(records_params, values_params);
    Ok(Self::write_send_await_stmt_prepare(cs, read_buffer, &(), cmd, stmts, stream).await?.0)
  }
}
