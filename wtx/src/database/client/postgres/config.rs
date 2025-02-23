use crate::{
  database::client::{postgres::PostgresError, rdbms::query_walker},
  misc::UriRef,
};

/// Configuration
#[derive(Debug, PartialEq, Eq)]
pub struct Config<'data> {
  pub(crate) application_name: &'data str,
  pub(crate) channel_binding: ChannelBinding,
  pub(crate) db: &'data str,
  pub(crate) password: &'data str,
  pub(crate) user: &'data str,
}

impl<'data> Config<'data> {
  /// Unwraps the elements from an URI.
  #[inline]
  pub fn from_uri(uri: &'data UriRef<'_>) -> crate::Result<Config<'data>> {
    let db = uri.path().get(1..).unwrap_or_default();
    let password = uri.password();
    let user = uri.user();
    let mut this =
      Self { application_name: "", channel_binding: ChannelBinding::Prefer, db, password, user };
    query_walker(uri, |key, value| this.set_param(key, value))?;
    Ok(this)
  }

  #[inline]
  fn set_param(&mut self, key: &str, value: &'data str) -> crate::Result<()> {
    match key {
      "application_name" => {
        self.application_name = value;
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

#[cfg(test)]
mod tests {
  use crate::{
    database::client::postgres::{Config, config::ChannelBinding},
    misc::Uri,
  };

  #[test]
  fn from_uri() {
    let uri = Uri::new("postgres://ab:cd@ef:5432/gh?application_name=ij&channel_binding=disable");
    let config = Config::from_uri(&uri).unwrap();
    assert_eq!(config.application_name, "ij");
    assert_eq!(config.channel_binding, ChannelBinding::Disable);
    assert_eq!(config.db, "gh");
    assert_eq!(config.password, "cd");
    assert_eq!(config.user, "ab");
  }
}
