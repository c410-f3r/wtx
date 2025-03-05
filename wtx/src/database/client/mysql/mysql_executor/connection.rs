use crate::{
  database::{
    Executor,
    client::mysql::{
      Config, ExecutorBuffer, MysqlError, MysqlExecutor,
      auth_plugin::AuthPlugin,
      capability::Capability,
      misc::{decode, fetch_msg, write_packet},
      mysql_executor::DFLT_PACKET_SIZE,
      mysql_protocol::{
        auth_switch_req::AuthSwitchReq, auth_switch_res::AuthSwitchRes,
        handshake_req::HandshakeReq, handshake_res::HandshakeRes, ok_res::OkRes,
      },
    },
  },
  misc::{
    ArrayVector, LeaseMut, Stream, Vector, from_utf8_basic,
    partitioned_filled_buffer::PartitionedFilledBuffer,
  },
};

impl<E, EB, S> MysqlExecutor<E, EB, S>
where
  E: From<crate::Error>,
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  #[inline]
  pub(crate) async fn connect0<'nb, IS>(
    config: &Config<'_>,
    net_buffer: &'nb mut PartitionedFilledBuffer,
    sequence_id: &mut u8,
    stream: &mut IS,
  ) -> Result<(u64, HandshakeRes<'nb>), E>
  where
    IS: Stream,
  {
    let mut capabilities: u64 = u64::from(Capability::DeprecateEof)
      | u64::from(Capability::FoundRows)
      | u64::from(Capability::IgnoreSpace)
      | u64::from(Capability::MultiResults)
      | u64::from(Capability::MultiStatements)
      | u64::from(Capability::OptionalResultsetMetadata)
      | u64::from(Capability::PluginAuth)
      | u64::from(Capability::PluginAuthLenencData)
      | u64::from(Capability::Protocol41)
      | u64::from(Capability::PsMultiResults)
      | u64::from(Capability::SecureConnection)
      | u64::from(Capability::Transactions);
    if config.db.is_some() {
      capabilities |= u64::from(Capability::ConnectWithDb);
    }
    let _ = fetch_msg(capabilities, net_buffer, sequence_id, stream).await?;
    let mut bytes = net_buffer._current();
    let res: HandshakeRes<'_> = decode(&mut bytes, ())?;
    capabilities &= res.capabilities;
    Ok((capabilities, res))
  }

  #[inline]
  pub(crate) async fn connect1(
    (capabilities, sequence_id): (&mut u64, &mut u8),
    config: &Config<'_>,
    encode_buffer: &mut Vector<u8>,
    handshake_res: &HandshakeRes<'_>,
    stream: &mut S,
  ) -> Result<(), E> {
    let tuple = (handshake_res.auth_plugin, config.password);
    let auth_response = if let (Some(plugin), Some(pw)) = tuple {
      Some(plugin.mask_pw(
        (&handshake_res.auth_plugin_data.0, handshake_res.auth_plugin_data.1),
        pw.as_bytes(),
      )?)
    } else {
      None
    };
    let handshake_req = HandshakeReq {
      auth_plugin: handshake_res.auth_plugin,
      auth_response,
      collation: config.collation,
      database: config.db,
      max_packet_size: DFLT_PACKET_SIZE,
      username: config.user,
    };
    write_packet((capabilities, sequence_id), encode_buffer, handshake_req, stream).await?;
    Ok(())
  }

  #[inline]
  pub(crate) async fn connect2<const IS_TLS: bool>(
    auth_plugin_data: ([u8; 8], ArrayVector<u8, 24>),
    (capabilities, sequence_id): (&mut u64, &mut u8),
    config: &Config<'_>,
    encode_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    mut plugin: Option<AuthPlugin>,
    stream: &mut S,
  ) -> Result<(), E> {
    loop {
      let _ = fetch_msg(*capabilities, net_buffer, sequence_id, stream).await?;
      let mut current = net_buffer._current();
      match current {
        [0, ..] => {
          let _: OkRes = decode(&mut current, ())?;
          break;
        }
        [254, ..] => {
          let other = config.enable_cleartext_plugin;
          let res_rslt: Result<AuthSwitchRes, _> = decode(&mut current, other);
          let res = res_rslt?;
          plugin = Some(res.auth_plugin);
          let bytes = res.auth_plugin.mask_pw(
            if let Some((lhs, rhs)) = &res.data {
              (lhs.as_slice(), rhs.as_slice())
            } else {
              (&[][..], &[][..])
            },
            config.password.unwrap_or_default().as_bytes(),
          )?;
          let payload = AuthSwitchReq(&bytes);
          write_packet((capabilities, sequence_id), encode_buffer, payload, stream).await?;
        }
        [a, rest @ ..] => {
          let (Some(plugin), Some(password)) = (plugin, &config.password) else {
            return Err(E::from(MysqlError::InvalidConnectionBytes.into()));
          };
          let [b, ..] = rest else {
            return Err(E::from(MysqlError::InvalidConnectionBytes.into()));
          };
          let is_auth_ok = plugin
            .manage_caching_sha2::<_, _, IS_TLS>(
              (auth_plugin_data.0, &auth_plugin_data.1),
              [*a, *b],
              (capabilities, sequence_id),
              encode_buffer,
              net_buffer,
              password,
              stream,
            )
            .await?;
          if is_auth_ok {
            break;
          };
        }
        _ => {
          return Err(E::from(MysqlError::InvalidConnectionBytes.into()));
        }
      }
    }
    Ok(())
  }

  #[inline]
  pub(crate) async fn connect3(&mut self, config: &Config<'_>) -> Result<(), E> {
    let mut buffer = Vector::new();
    buffer.extend_from_copyable_slice("SET ".as_bytes())?;
    let sql_mode_opt = match (config.no_engine_substitution, config.pipes_as_concat) {
      (false, false) => None,
      (false, true) => Some(&b"sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT')),"[..]),
      (true, false) => {
        Some(&b"sql_mode=(SELECT CONCAT(@@sql_mode, ',NO_ENGINE_SUBSTITUTION')),"[..])
      }
      (true, true) => Some(
        &b"sql_mode=(SELECT CONCAT(@@sql_mode, ',NO_ENGINE_SUBSTITUTION,PIPES_AS_CONCAT')),"[..],
      ),
    };
    if let Some(sql_mode) = sql_mode_opt {
      buffer.extend_from_copyable_slice(sql_mode)?;
    }
    if let Some(timezone) = &config.timezone {
      let _ = buffer.extend_from_copyable_slices([b"time_zone=", timezone.as_bytes(), b"',"])?;
    }
    if config.set_names {
      let _ = buffer.extend_from_copyable_slices([
        "NAMES ".as_bytes(),
        config.charset.strings().ident.as_bytes(),
        b" COLLATE ",
        config.collation.strings().ident.as_bytes(),
        b",",
      ])?;
    }
    if buffer.len() > 4 {
      let _ = buffer.pop();
      self.execute(from_utf8_basic(&buffer).map_err(crate::Error::from)?, |_| Ok(())).await?;
    }
    buffer.clear();
    Ok(())
  }
}
