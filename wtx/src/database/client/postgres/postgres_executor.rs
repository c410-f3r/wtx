mod authentication;
mod fetch;
mod simple_query;
mod stmt;

use crate::{
  collection::TryExtend,
  database::{
    Database, Executor, RecordValues, StmtCmd,
    client::{
      postgres::{
        Batch, Config, Postgres, PostgresError, PostgresRecords,
        executor_buffer::ExecutorBuffer,
        message::MessageTy,
        misc::data_row,
        protocol::{encrypted_conn, initial_conn_msg},
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
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
  E: From<crate::Error>,
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

  /// See [`Batch`].
  #[inline]
  pub fn batch(&mut self) -> Batch<'_, E, EB, S> {
    Batch::new(self)
  }

  /// Mutable buffer reference
  #[inline]
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

  async fn execute_many<'this, B>(
    &'this mut self,
    buffer: &mut B,
    cmd: &str,
    cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<(), E>
  where
    B: TryExtend<[<Self::Database as Database>::Records<'this>; 1]>,
  {
    let ExecutorBuffer { common, .. } = self.eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params, .. } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Self::simple_query_execute(
      buffer,
      cmd,
      &mut self.cs,
      net_buffer,
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
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let tuple = Self::write_send_await_stmt_prepare(cs, net_buffer, sc, stmts, stream, &[]).await?;
    let (_, stmt_cmd_id_array, stmt_mut) = tuple;
    Self::write_send_await_stmt_bind(cs, net_buffer, rv, &stmt_cmd_id_array, stream).await?;
    let begin = net_buffer.current_end_idx();
    let begin_data = net_buffer.current_end_idx().wrapping_add(7);
    loop {
      let msg = Self::fetch_msg_from_stream(cs, net_buffer, stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        MessageTy::DataRow(values_len) => {
          data_row(
            begin,
            begin_data,
            net_buffer,
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
          return Err(<_>::from(
            PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into(),
          ));
        }
      }
    }
    Ok(PostgresRecords::new(
      net_buffer.all().get(begin_data..net_buffer.current_end_idx()).unwrap_or_default(),
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
    let Self { cs, eb, phantom: _, stream } = self;
    let ExecutorBuffer { common, .. } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Ok(Self::write_send_await_stmt_prepare(cs, net_buffer, cmd, stmts, stream, &[]).await?.0)
  }
}
