use crate::misc::{atoi, str_split1, UriRef};
use core::time::Duration;

/// Configuration
#[derive(Debug, PartialEq, Eq)]
pub struct Config<'data> {
  pub(crate) app_name: &'data str,
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
      connect_timeout: Duration::ZERO,
      db: uri.path().get(1..).unwrap_or_default(),
      host: uri.host(),
      keepalives: true,
      load_balance_hosts: LoadBalanceHosts::Disable,
      password: uri.password(),
      port: atoi(uri.port().as_bytes())?,
      target_session_attrs: TargetSessionAttrs::Any,
      tcp_user_timeout: Duration::ZERO,
      user: uri.user(),
    };
    for key_value in str_split1(uri.query(), b'&') {
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
      "connect_timeout" => {
        if let Ok(timeout) = atoi(value.as_bytes()) {
          self.connect_timeout = Duration::from_secs(timeout);
        }
      }
      "load_balance_hosts" => {
        self.load_balance_hosts = match value {
          "disable" => LoadBalanceHosts::Disable,
          "random" => LoadBalanceHosts::Random,
          _ => return Err(crate::Error::UnknownConfigurationParameter),
        };
      }
      "target_session_attrs" => {
        self.target_session_attrs = match value {
          "any" => TargetSessionAttrs::Any,
          "read-write" => TargetSessionAttrs::ReadWrite,
          _ => return Err(crate::Error::UnknownConfigurationParameter),
        };
      }
      "tcp_user_timeout" => {
        if let Ok(timeout) = atoi(value.as_bytes()) {
          self.tcp_user_timeout = Duration::from_secs(timeout);
        }
      }
      _ => return Err(crate::Error::UnknownConfigurationParameter),
    }
    Ok(())
  }
}

/// Load balancing configuration.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum LoadBalanceHosts {
  /// Make connection attempts to hosts in the order provided.
  Disable,
  /// Make connection attempts to hosts in a random order.
  Random,
}

/// Properties required of a session.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum TargetSessionAttrs {
  /// No special properties are required.
  Any,
  /// The session must allow writes.
  ReadWrite,
}
