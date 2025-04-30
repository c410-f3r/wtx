mod connection;
mod fetch;
mod prepare;
mod simple_query;

use crate::{
  database::{
    Database, DatabaseError, Executor, RecordValues, StmtCmd,
    client::{
      mysql::{
        Config, ExecutorBuffer, Mysql, MysqlError, MysqlRecord, MysqlRecords,
        capability::Capability, misc::write_packet, mysql_protocol::initial_req::InitialReq,
      },
      rdbms::{clear_cmd_buffers, common_executor_buffer::CommonExecutorBuffer},
    },
  },
  misc::{ConnectionState, DEController, LeaseMut},
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
  pub async fn connect(config: &Config<'_>, mut eb: EB, mut stream: S) -> Result<Self, E> {
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
    Self::connect2::<false>(
      (handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1.try_into()?),
      (&mut capabilities, &mut sequence_id),
      config,
      encode_buffer,
      net_buffer,
      plugin,
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
    _rng: &mut RNG,
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
      write_packet((&mut capabilities, &mut sequence_id), encode_buffer, req, &mut stream).await?;
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
    Self::connect2::<true>(
      (handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1.try_into()?),
      (&mut capabilities, &mut sequence_id),
      config,
      encode_buffer,
      net_buffer,
      plugin,
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
  async fn execute(
    &mut self,
    cmd: &str,
    cb: impl FnMut(u64) -> Result<(), <Self::Database as DEController>::Error>,
  ) -> Result<(), <Self::Database as DEController>::Error> {
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, values_params, .. } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Self::simple_query_execute(
      (capabilities, sequence_id),
      cmd,
      encode_buffer,
      net_buffer,
      records_params,
      stream,
      values_params,
      cb,
    )
    .await?;
    Ok(())
  }

  #[inline]
  async fn execute_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> Result<u64, <Self::Database as DEController>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let mut rows: u64 = 0;
    let _ = Self::write_send_await_stmt::<_, _, false>(
      (capabilities, sequence_id),
      encode_buffer,
      net_buffer,
      records_params,
      rv,
      sc,
      stmts,
      stream,
      values_params,
      |local_rows| {
        rows = rows.wrapping_add(local_rows);
        Ok(())
      },
      |_| Ok(()),
    )
    .await?;
    Ok(rows)
  }

  #[inline]
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
    let Self { capabilities, cs: _, eb, sequence_id, stream, .. } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let (start, stmt) = Self::write_send_await_stmt::<_, _, false>(
      (capabilities, sequence_id),
      encode_buffer,
      net_buffer,
      records_params,
      rv,
      sc,
      stmts,
      stream,
      values_params,
      |_| Ok(()),
      |record| cb(&record),
    )
    .await?;
    Ok(MysqlRecords::new(
      net_buffer._all().get(start..).unwrap_or_default(),
      records_params,
      stmt,
      values_params,
    ))
  }

  #[inline]
  async fn fetch_with_stmt<SC, RV>(
    &mut self,
    sc: SC,
    rv: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    let Self { capabilities, cs: _, eb, sequence_id, stream, .. } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    let (start, stmt) = Self::write_send_await_stmt::<_, _, true>(
      (capabilities, sequence_id),
      encode_buffer,
      net_buffer,
      records_params,
      rv,
      sc,
      stmts,
      stream,
      values_params,
      |_| Ok(()),
      |_| Ok(()),
    )
    .await?;
    let Some(record @ [_, ..]) =
      net_buffer._all().get(start..).and_then(|el| el.get(records_params.first()?.0.clone()))
    else {
      return Err(crate::Error::from(DatabaseError::MissingRecord).into());
    };
    Ok(MysqlRecord::new(record, stmt, values_params))
  }

  #[inline]
  async fn prepare(&mut self, cmd: &str) -> Result<u64, E> {
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { common, encode_buffer } = eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, records_params, stmts, values_params } = common;
    clear_cmd_buffers(net_buffer, records_params, values_params);
    Ok(
      Self::write_send_await_stmt_prot(
        (capabilities, sequence_id),
        encode_buffer,
        net_buffer,
        cmd,
        stmts,
        stream,
        |_| Ok(()),
      )
      .await?
      .0,
    )
  }
}
