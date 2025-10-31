use crate::{
  database::client::mysql::{
    MysqlError,
    auth_plugin::AuthPlugin,
    capability::Capability,
    protocol::{Protocol, decode_wrapper_protocol::DecodeWrapperProtocol},
    status::Status,
  },
  de::{Decode, FromRadix10},
  misc::{bytes_split_once1, bytes_split1},
};

#[derive(Debug)]
pub(crate) struct HandshakeRes<'bytes> {
  pub(crate) auth_plugin: Option<AuthPlugin>,
  pub(crate) auth_plugin_data: ([u8; 8], &'bytes [u8]),
  pub(crate) capabilities: u64,
}

impl<'de, DO, E> Decode<'de, Protocol<DO, E>> for HandshakeRes<'de>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'de, '_, DO>) -> Result<Self, E> {
    let [_protocol_version, rest0 @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidHandshakeBytes.into()));
    };
    let Some((server_version, rest1)) = bytes_split_once1(rest0, b'\0') else {
      return Err(E::from(MysqlError::InvalidHandshakeBytes.into()));
    };
    let [a, b, c, d, e, f, g, h, i, j, k, l, _, rest2 @ ..] = rest1 else {
      return Err(E::from(MysqlError::InvalidHandshakeBytes.into()));
    };
    let _connection_id = u32::from_le_bytes([*a, *b, *c, *d]);
    let auth_plugin_data0 = [*e, *f, *g, *h, *i, *j, *k, *l];

    let [a, b, _default_collation, d, e, f, g, h, _, _, _, _, _, _, i, j, k, l, rest3 @ ..] = rest2
    else {
      return Err(E::from(MysqlError::InvalidHandshakeBytes.into()));
    };

    let mut capabilities: u64 = u64::from(u16::from_le_bytes([*a, *b]));

    let _status = Status::try_from(u16::from_le_bytes([*d, *e]))?;
    capabilities |= u64::from(u16::from_le_bytes([*f, *g])) << 16;

    let plugin_auth_n = u64::from(Capability::PluginAuth);
    let auth_plugin_data_len = if capabilities & plugin_auth_n == plugin_auth_n { *h } else { 0 };

    let mysql_n = u64::from(Capability::Mysql);
    if capabilities & mysql_n == mysql_n {
      capabilities |= u64::from(u32::from_le_bytes([*i, *j, *k, *l])) << 32;
    }

    let secure_connection_n = u64::from(Capability::SecureConnection);
    let (auth_plugin_data1, rest4) = if capabilities & secure_connection_n == secure_connection_n {
      let len = auth_plugin_data_len.saturating_sub(9).max(12);
      let Some((auth_plugin_data1, rest4)) = rest3.split_at_checked(len.into()) else {
        return Err(E::from(MysqlError::InvalidHandshakeBytes.into()));
      };
      (auth_plugin_data1, rest4)
    } else {
      (&[][..], rest3)
    };

    let auth_plugin = if capabilities & plugin_auth_n == plugin_auth_n {
      let auth_plugin_bytes = if let [_, elem @ .., _] = rest4 { elem } else { rest4 };
      Some(AuthPlugin::try_from(auth_plugin_bytes)?)
    } else {
      None
    };

    let _server_version = server_version_array(server_version);

    Ok(Self { auth_plugin, auth_plugin_data: (auth_plugin_data0, auth_plugin_data1), capabilities })
  }
}

fn server_version_array(bytes: &[u8]) -> [u16; 3] {
  let mut iter = bytes_split1(bytes, b'.');
  let major = iter.next().and_then(|el| FromRadix10::from_radix_10(el).ok()).unwrap_or_default();
  let minor = iter.next().and_then(|el| FromRadix10::from_radix_10(el).ok()).unwrap_or_default();
  let patch = iter.next().and_then(|el| FromRadix10::from_radix_10(el).ok()).unwrap_or_default();
  [major, minor, patch]
}
