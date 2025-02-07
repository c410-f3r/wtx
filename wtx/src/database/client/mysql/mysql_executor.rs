use crate::{
  database::{
    Database, RecordValues, StmtCmd,
    client::mysql::{
      Config, ExecutorBuffer, Mysql, capability::Capability, handshake_req::HandshakeReq,
      handshake_res::HandshakeRes,
    },
  },
  misc::{
    _read_header, _read_payload, ConnectionState, DEController, LeaseMut, Stream, Usize,
    partitioned_filled_buffer::PartitionedFilledBuffer,
  },
};
use core::marker::PhantomData;

/// Executor
#[derive(Debug)]
pub struct MysqlExecutor<E, EB, S> {
  pub(crate) cs: ConnectionState,
  pub(crate) eb: EB,
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stream: S,
}

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub async fn connect(config: &Config<'_>, mut eb: EB, mut stream: S) -> crate::Result<Self> {
    let mut capabilities: u64 = u64::from(Capability::DeprecateEof)
      | u64::from(Capability::FoundRows)
      | u64::from(Capability::IgnoreSpace)
      | u64::from(Capability::MultiResults)
      | u64::from(Capability::MultiStatements)
      | u64::from(Capability::PluginAuth)
      | u64::from(Capability::PluginAuthLenencData)
      | u64::from(Capability::Protocol41)
      | u64::from(Capability::PsMultiResults)
      | u64::from(Capability::SecureConnection)
      | u64::from(Capability::Ssl)
      | u64::from(Capability::Transactions);

    if config.database.is_some() {
      capabilities |= u64::from(Capability::ConnectWithDb);
    }

    Self::fetch_msg(&mut eb.lease_mut().nb, &mut stream).await?;
    let handshake = HandshakeRes::try_from(eb.lease_mut().nb._current())?;

    capabilities &= handshake.capabilities;
    capabilities |= u64::from(Capability::Protocol41);

    //let auth_response = if let (Some(plugin), Some(password)) = (handshake.auth_plugin, config.password) {
    //  Some(plugin.scramble(&mut stream, password, &nonce).await?)
    //} else {
    //    None
    //};

    //HandshakeReq {
    //  auth_plugin: todo!(),
    //  auth_response: todo!(),
    //  collation: todo!(),
    //  database: todo!(),
    //  max_packet_size: todo!(),
    //  username: todo!(),
    //}
    //.write()
    //.await?;

    Ok(Self { cs: ConnectionState::Open, eb, phantom: PhantomData, stream })
  }

  #[inline]
  pub async fn connect_encrypted(
    config: &Config<'_>,
    mut eb: EB,
    mut stream: S,
  ) -> crate::Result<Self> {
    todo!();
  }

  async fn fetch_msg(pfb: &mut PartitionedFilledBuffer, stream: &mut S) -> crate::Result<()> {
    let mut len = Self::fetch_one_msg(pfb, stream).await?;
    while len == 0xFF_FF_FF {
      len = Self::fetch_one_msg(pfb, stream).await?;
    }
    Ok(())
  }

  async fn fetch_one_msg(
    pfb: &mut PartitionedFilledBuffer,
    stream: &mut S,
  ) -> crate::Result<usize> {
    pfb._reserve(4)?;
    let mut read = pfb._following_len();
    let buffer = pfb._following_rest_mut();
    let [a0, b0, c0, _] = _read_header::<0, 4, S>(buffer, &mut read, stream).await?;
    let len = Usize::from(u32::from_le_bytes([a0, b0, c0, 0])).into_usize();
    _read_payload((4, len), pfb, &mut read, stream).await?;
    Ok(len)
  }
}

impl<E, EB, S> crate::database::Executor for MysqlExecutor<E, EB, S>
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
  async fn execute(&mut self, _: &str, _: impl FnMut(u64)) -> crate::Result<()> {
    todo!()
  }

  #[inline]
  async fn execute_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
  ) -> Result<u64, <Self::Database as DEController>::Error>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    todo!()
  }

  #[inline]
  async fn fetch_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    todo!()
  }

  #[inline]
  async fn fetch_many_with_stmt<SC, RV>(
    &mut self,
    _: SC,
    _: RV,
    _: impl FnMut(&<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<<Self::Database as Database>::Records<'_>, E>
  where
    RV: RecordValues<Self::Database>,
    SC: StmtCmd,
  {
    todo!()
  }

  #[inline]
  async fn prepare(&mut self, _: &str) -> Result<u64, E> {
    todo!()
  }
}
