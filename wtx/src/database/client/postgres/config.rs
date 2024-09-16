use crate::{
  database::client::postgres::PostgresError,
  misc::{into_rslt, str_split1, FromRadix10, UriRef},
};
use core::time::Duration;

/// Configuration
#[derive(Debug, PartialEq, Eq)]
pub struct Config<'data> {
  pub(crate) app_name: &'data str,
  pub(crate) channel_binding: ChannelBinding,
  pub(crate) connect_timeout: Duration,
  pub(crate) db: &'data str,
  pub(crate) host: &'data str,
  pub(crate) keepalives: bool,
  pub(crate) load_balance_hosts: LoadBalanceHosts,
  pub(crate) password: &'data str,
  pub(crate) port: u16,
  pub(crate) target_session_attrs: TargetSessionAttrs,
  pub(crate) tcp_user_timeout: Duration,
  pub(crate) user: &'data str,
}

impl<'data> Config<'data> {
  /// Unwraps the elements of an URI.
  #[inline]
  pub fn from_uri(uri: &'data UriRef<'_>) -> crate::Result<Config<'data>> {
    let mut this = Self {
      app_name: "",
      channel_binding: ChannelBinding::Prefer,
      connect_timeout: Duration::ZERO,
      db: uri.relative_reference().get(1..).unwrap_or_default(),
      host: uri.host(),
      keepalives: true,
      load_balance_hosts: LoadBalanceHosts::Disable,
      password: uri.password(),
      port: into_rslt(uri.port())?,
      target_session_attrs: TargetSessionAttrs::Any,
      tcp_user_timeout: Duration::ZERO,
      user: uri.user(),
    };
    for key_value in str_split1(uri.query_and_fragment(), b'&') {
      let mut iter = str_split1(key_value, b':');
      if let [Some(key), Some(value)] = [iter.next(), iter.next()] {
        this.set_param(key, value)?;
      }
    }
    Ok(this)
  }

  fn set_param(&mut self, key: &str, value: &'data str) -> crate::Result<()> {
    match key {
      "application_name" => {
        self.app_name = value;
      }
      "channel_binding" => {
        let channel_binding = match value {
          "disable" => ChannelBinding::Disable,
          "prefer" => ChannelBinding::Prefer,
          "require" => ChannelBinding::Require,
          _ => return Err(PostgresError::UnknownConfigurationParameter.into()),
        };
        self.channel_binding = channel_binding;
      }
      "connect_timeout" => {
        if let Ok(timeout) = u64::from_radix_10(value.as_bytes()) {
          self.connect_timeout = Duration::from_secs(timeout);
        }
      }
      "load_balance_hosts" => {
        self.load_balance_hosts = match value {
          "disable" => LoadBalanceHosts::Disable,
          "random" => LoadBalanceHosts::Random,
          _ => return Err(PostgresError::UnknownConfigurationParameter.into()),
        };
      }
      "target_session_attrs" => {
        self.target_session_attrs = match value {
          "any" => TargetSessionAttrs::Any,
          "read-write" => TargetSessionAttrs::ReadWrite,
          _ => return Err(PostgresError::UnknownConfigurationParameter.into()),
        };
      }
      "tcp_user_timeout" => {
        if let Ok(timeout) = u64::from_radix_10(value.as_bytes()) {
          self.tcp_user_timeout = Duration::from_secs(timeout);
        }
      }
      _ => return Err(PostgresError::UnknownConfigurationParameter.into()),
    }
    Ok(())
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChannelBinding {
  Disable,
  Prefer,
  Require,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LoadBalanceHosts {
  Disable,
  Random,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TargetSessionAttrs {
  Any,
  ReadWrite,
}
