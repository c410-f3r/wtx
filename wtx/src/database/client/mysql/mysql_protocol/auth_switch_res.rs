use crate::{
  database::client::mysql::{
    MysqlError,
    auth_plugin::AuthPlugin,
    mysql_protocol::{MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol},
  },
  de::Decode,
  misc::bytes_split_once1,
};

pub(crate) struct AuthSwitchRes {
  pub(crate) auth_plugin: AuthPlugin,
  pub(crate) data: Option<([u8; 8], [u8; 12])>,
}

impl<E> Decode<'_, MysqlProtocol<bool, E>> for AuthSwitchRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(_: &mut (), dw: &mut DecodeWrapperProtocol<'_, '_, bool>) -> Result<Self, E> {
    let [a, rest0 @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidAuthSwitchBytes.into()));
    };
    if *a != 254 {
      return Err(E::from(MysqlError::InvalidAuthSwitchBytes.into()));
    }
    let Some((auth_plugin_bytes, rest1)) = bytes_split_once1(rest0, b'\0') else {
      return Err(E::from(MysqlError::InvalidAuthSwitchBytes.into()));
    };
    let auth_plugin = AuthPlugin::try_from(auth_plugin_bytes)?;
    if matches!(auth_plugin, AuthPlugin::MysqlClear) && dw.other {
      return Err(E::from(MysqlError::InvalidAuthSwitchBytes.into()));
    }
    if matches!(auth_plugin, AuthPlugin::MysqlClear) && rest1.is_empty() {
      return Ok(Self { auth_plugin, data: None });
    }
    let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, _u] = rest1 else {
      return Err(E::from(MysqlError::InvalidAuthSwitchBytes.into()));
    };
    Ok(Self {
      auth_plugin,
      data: Some((
        [*a, *b, *c, *d, *e, *f, *g, *h],
        [*i, *j, *k, *l, *m, *n, *o, *p, *q, *r, *s, *t],
      )),
    })
  }
}
