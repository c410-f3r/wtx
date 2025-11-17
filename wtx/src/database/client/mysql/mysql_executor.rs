mod connection;
mod fetch;
mod prepare;
mod simple_query;

use crate::{
  collection::TryExtend,
  database::{
    Database, Executor, RecordValues, StmtCmd,
    client::{
      mysql::{
        Config, ExecutorBuffer, Mysql, MysqlError, MysqlRecords,
        capability::Capability,
        misc::{fetch_msg, send_packet, write_and_send_packet},
        protocol::{initial_req::InitialReq, ping_req::PingReq, stmt_execute_req::StmtExecuteReq},
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
  misc::{ConnectionState, LeaseMut},
  rng::CryptoRng,
  stream::{Stream, StreamWithTls},
};
use core::marker::PhantomData;

pub(crate) const DFLT_PACKET_SIZE: u32 = 1024 * 1024 * 4;
pub(crate) const MAX_PAYLOAD: u32 = 16_777_215;

/// Executor
#[derive(Debug)]
pub struct MysqlExecutor<E, EB, S> {
  pub(crate) capabilities: u64,
  pub(crate) cs: ConnectionState,
  pub(crate) eb: EB,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) sequence_id: u8,
  pub(crate) stream: S,
}

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  /// Connects with an unencrypted stream.
  #[inline]
  pub async fn connect<RNG>(
    config: &Config<'_>,
    mut eb: EB,
    rng: &mut RNG,
    mut stream: S,
  ) -> Result<Self, E>
  where
    RNG: CryptoRng,
  {
    eb.lease_mut().clear();
    let mut sequence_id = 0;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, .. } = common;
    let tuple = Self::connect0(config, net_buffer, &mut sequence_id, &mut stream).await?;
    let (mut capabilities, handshake_res) = tuple;
    let plugin = handshake_res.auth_plugin;
    Self::connect1(
      (&mut capabilities, &mut sequence_id),
      config,
      encode_buffer,
      &handshake_res,
      &mut stream,
    )
    .await?;
    Self::connect2::<_, false>(
      (handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1.try_into()?),
      (&mut capabilities, &mut sequence_id),
      config,
      encode_buffer,
      net_buffer,
      plugin,
      rng,
      &mut stream,
    )
    .await?;
    let mut this = Self {
      capabilities,
      cs: ConnectionState::Open,
      eb,
      phantom: PhantomData,
      sequence_id,
      stream,
    };
    this.connect3(config).await?;
    Ok(this)
  }

  /// Initially connects with an unencrypted stream that should be later upgraded to an encrypted
  /// stream.
  #[inline]
  pub async fn connect_encrypted<F, IS, RNG>(
    config: &Config<'_>,
    mut eb: EB,
    rng: &mut RNG,
    mut stream: IS,
    cb: impl FnOnce(IS) -> F,
  ) -> Result<Self, E>
  where
    F: Future<Output = crate::Result<S>>,
    IS: Stream,
    RNG: CryptoRng,
    S: StreamWithTls,
  {
    eb.lease_mut().clear();
    let mut sequence_id = 0;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, .. } = common;
    let tuple = Self::connect0(config, net_buffer, &mut sequence_id, &mut stream).await?;
    let (mut capabilities, handshake_res) = tuple;
    {
      let ssl_n = u64::from(Capability::Ssl);
      if handshake_res.capabilities & ssl_n == 0 {
        return Err(crate::Error::from(MysqlError::UnsupportedServerSsl).into());
      }
      capabilities |= ssl_n;
      let req = InitialReq { collation: config.collation, max_packet_size: DFLT_PACKET_SIZE };
      write_and_send_packet((&mut capabilities, &mut sequence_id), encode_buffer, req, &mut stream)
        .await?;
    }
    let mut enc_stream = cb(stream).await?;
    let plugin = handshake_res.auth_plugin;
    Self::connect1(
      (&mut capabilities, &mut sequence_id),
      config,
      encode_buffer,
      &handshake_res,
      &mut enc_stream,
    )
    .await?;
    Self::connect2::<_, true>(
      (handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1.try_into()?),
      (&mut capabilities, &mut sequence_id),
      config,
      encode_buffer,
      net_buffer,
      plugin,
      rng,
      &mut enc_stream,
    )
    .await?;
    let mut this = Self {
      capabilities,
      cs: ConnectionState::Open,
      eb,
      phantom: PhantomData,
      sequence_id,
      stream: enc_stream,
    };
    this.connect3(config).await?;
    Ok(this)
  }
}

impl<E, EB, S> Executor for MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  type Database = Mysql<E>;

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
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Self::simple_query_execute(
      buffer,
      (capabilities, sequence_id),
      cmd,
      encode_buffer,
      net_buffer,
      records_params,
      stmts,
      stream,
      values_params,
      cb,
    )
    .await
  }

  #[inline]
  async fn execute_with_stmt_many<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
    cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<<Self::Database as Database>::Records<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { capabilities, cs: _, eb, sequence_id, stream, .. } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let (_, _, mut stmt_mut) = Self::write_send_await_stmt(
      (capabilities, sequence_id),
      encode_buffer,
      net_buffer,
      sc,
      stmts,
      stream,
      rv.len(),
    )
    .await?;
    let mut tys = stmt_mut.tys_mut().iter_mut();
    rv.walk(|_, ty_opt| {
      if let (Some(ty), Some(value)) = (ty_opt, tys.next()) {
        value.1 = ty;
      }
      Ok(())
    })?;
    send_packet(
      (capabilities, sequence_id),
      encode_buffer,
      StmtExecuteReq { rv, stmt_id: stmt_mut.aux, tys: stmt_mut.tys() },
      stream,
    )
    .await?;
    let start = net_buffer.current_end_idx();
    Self::fetch_bin_cmd(
      *capabilities,
      net_buffer,
      records_params,
      sequence_id,
      &mut stmt_mut,
      stream,
      values_params,
      cb,
    )
    .await?;
    Ok(MysqlRecords::new(
      net_buffer.all().get(start..).unwrap_or_default(),
      records_params,
      stmt_mut.into_stmt(),
      values_params,
    ))
  }

  #[inline]
  async fn ping(&mut self) -> Result<(), E> {
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, values_params, .. } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    send_packet::<E, _, _>((capabilities, sequence_id), encode_buffer, PingReq, stream).await?;
    let _ = fetch_msg(*capabilities, net_buffer, sequence_id, stream).await?;
    Ok(())
  }

  #[inline]
  async fn prepare(&mut self, cmd: &str) -> Result<u64, E> {
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Ok(
      Self::write_send_await_stmt(
        (capabilities, sequence_id),
        encode_buffer,
        net_buffer,
        cmd,
        stmts,
        stream,
        0,
      )
      .await?
      .0,
    )
  }
}
