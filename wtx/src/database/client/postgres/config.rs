use crate::{
  database::client::postgres::PostgresError,
  misc::{UriRef, str_split_once1, str_split1},
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
    let mut this = Self {
      application_name: "",
      channel_binding: ChannelBinding::Prefer,
      db: uri.path().get(1..).unwrap_or_default(),
      password: uri.password(),
      user: uri.user(),
    };
    let mut pair_iter = str_split1(uri.query_and_fragment(), b'&');
    if let Some(mut key_value) = pair_iter.next() {
      key_value = key_value.get(1..).unwrap_or_default();
      if let Some((key, value)) = str_split_once1(key_value, b'=') {
        this.set_param(key, value)?;
      }
    }
    for key_value in pair_iter {
      if let Some((key, value)) = str_split_once1(key_value, b'=') {
        this.set_param(key, value)?;
      }
    }
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
