mod connection;
mod fetch;
mod prepare;
mod simple_query;

use crate::{
  database::{
    Database, Executor, RecordValues, Records, StmtCmd,
    client::mysql::{
      Config, ExecutorBuffer, Mysql, MysqlRecord, MysqlRecords, capability::Capability,
      misc::write_packet, mysql_protocol::initial_req::InitialReq,
    },
  },
  misc::{ConnectionState, DEController, LeaseMut, Stream, StreamWithTls, Usize},
};
use core::marker::PhantomData;

pub(crate) const DFLT_PACKET_SIZE: u32 = 1024;
pub(crate) const MAX_PAYLOAD: u32 = 16777215;

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
    let ExecutorBuffer { enc_buffer, net_buffer, .. } = eb.lease_mut();
    let tuple = Self::connect0(config, net_buffer, &mut sequence_id, &mut stream).await?;
    let (mut capabilities, handshake_res) = tuple;
    let plugin = handshake_res.auth_plugin;
    Self::connect1(
      (&mut capabilities, &mut sequence_id),
      config,
      enc_buffer,
      &handshake_res,
      &mut stream,
    )
    .await?;
    Self::connect2::<false>(
      (handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1.try_into()?),
      (&mut capabilities, &mut sequence_id),
      config,
      enc_buffer,
      net_buffer,
      plugin,
      &mut stream,
    )
    .await?;
    Self::connect3(enc_buffer, config)?;
    Ok(Self {
      capabilities,
      cs: ConnectionState::Open,
      eb,
      phantom: PhantomData,
      sequence_id,
      stream,
    })
  }

  /// Initially connects with an unencrypted stream that should be later upgraded to an encrypted
  /// stream.
  #[inline]
  pub async fn connect_encrypted<F, IS, RNG>(
    config: &Config<'_>,
    mut eb: EB,
    mut stream: IS,
    cb: impl FnOnce(IS) -> F,
  ) -> Result<Self, E>
  where
    F: Future<Output = crate::Result<S>>,
    IS: Stream,
    S: StreamWithTls,
  {
    eb.lease_mut().clear();
    let mut sequence_id = 0;
    let ExecutorBuffer { enc_buffer, net_buffer, .. } = eb.lease_mut();
    let tuple = Self::connect0(config, net_buffer, &mut sequence_id, &mut stream).await?;
    let (mut capabilities, handshake_res) = tuple;
    {
      capabilities |= u64::from(Capability::Ssl);
      let req = InitialReq { collation: config.collation, max_packet_size: DFLT_PACKET_SIZE };
      write_packet((&mut capabilities, &mut sequence_id), enc_buffer, req, &mut stream).await?;
      enc_buffer.clear();
    }
    let mut enc_stream = cb(stream).await?;
    let plugin = handshake_res.auth_plugin;
    Self::connect1(
      (&mut capabilities, &mut sequence_id),
      config,
      enc_buffer,
      &handshake_res,
      &mut enc_stream,
    )
    .await?;
    Self::connect2::<true>(
      (handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1.try_into()?),
      (&mut capabilities, &mut sequence_id),
      config,
      enc_buffer,
      net_buffer,
      plugin,
      &mut enc_stream,
    )
    .await?;
    Self::connect3(enc_buffer, config)?;
    Ok(Self {
      capabilities,
      cs: ConnectionState::Open,
      eb,
      phantom: PhantomData,
      sequence_id,
      stream: enc_stream,
    })
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
    let ExecutorBuffer { enc_buffer, net_buffer, stmts: _, vb } = eb.lease_mut();
    Self::simple_query_execute(
      (capabilities, sequence_id),
      cmd,
      enc_buffer,
      net_buffer,
      stream,
      vb,
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
    Ok(Usize::from(self.fetch_many_with_stmt(sc, rv, |_| Ok(())).await?.len()).into_u64())
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
    let ExecutorBuffer { enc_buffer, net_buffer, stmts, vb, .. } = eb.lease_mut();
    let mut record_len_opt = None;
    let stmt = Self::write_send_await_stmt::<_, _, true>(
      (capabilities, sequence_id),
      enc_buffer,
      net_buffer,
      rv,
      sc,
      stmts,
      stream,
      vb,
      |local_record| {
        record_len_opt = Some(local_record.bytes.len());
        Ok(())
      },
    )
    .await?;
    let Some(record_len) = record_len_opt else {
      panic!();
    };
    Ok(MysqlRecord {
      bytes: enc_buffer.get(..record_len).unwrap_or_default(),
      phantom: PhantomData,
      stmt,
      values_bytes_offsets: vb.get(0..record_len).unwrap_or_default(),
    })
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
    let ExecutorBuffer { enc_buffer, net_buffer, stmts, vb, .. } = eb.lease_mut();
    let stmt = Self::write_send_await_stmt::<_, _, false>(
      (capabilities, sequence_id),
      enc_buffer,
      net_buffer,
      rv,
      sc,
      stmts,
      stream,
      vb,
      |record| cb(&record),
    )
    .await?;
    Ok(MysqlRecords::new(&[], &[], stmt.clone(), vb))
  }

  #[inline]
  async fn prepare(&mut self, cmd: &str) -> Result<u64, E> {
    let Self { capabilities, cs: _, eb, phantom: _, sequence_id, stream } = self;
    let ExecutorBuffer { enc_buffer, net_buffer, stmts, .. } = eb.lease_mut();
    Ok(
      Self::write_send_await_stmt_prot(
        (capabilities, sequence_id),
        enc_buffer,
        net_buffer,
        cmd,
        stmts,
        stream,
      )
      .await?
      .0,
    )
  }
}
