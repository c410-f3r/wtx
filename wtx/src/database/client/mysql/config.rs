use crate::{
  database::client::{
    mysql::{MysqlError, charset::Charset, collation::Collation},
    rdbms::query_walker,
  },
  misc::UriRef,
};

/// Configuration
#[derive(Debug)]
pub struct Config<'data> {
  pub(crate) charset: Charset,
  pub(crate) collation: Collation,
  pub(crate) db: Option<&'data str>,
  pub(crate) enable_cleartext_plugin: bool,
  pub(crate) no_engine_substitution: bool,
  pub(crate) password: Option<&'data str>,
  pub(crate) pipes_as_concat: bool,
  pub(crate) set_names: bool,
  pub(crate) timezone: Option<&'data str>,
  pub(crate) user: &'data str,
}

impl<'data> Config<'data> {
  /// Unwraps the elements from an URI.
  #[inline]
  pub fn from_uri(uri: &'data UriRef<'_>) -> crate::Result<Config<'data>> {
    let db = uri.path().get(1..).unwrap_or_default();
    let password = uri.password();
    let user = uri.user();
    let mut this = Self {
      charset: Charset::utf8mb4,
      collation: Charset::utf8mb4.default_collation(),
      db: if db.is_empty() { None } else { Some(db) },
      enable_cleartext_plugin: false,
      no_engine_substitution: true,
      password: if password.is_empty() { None } else { Some(password) },
      pipes_as_concat: true,
      set_names: true,
      timezone: None,
      user,
    };
    query_walker(uri, |key, value| this.set_param(key, value))?;
    Ok(this)
  }

  #[inline]
  fn set_param(&mut self, key: &str, value: &'data str) -> crate::Result<()> {
    match key {
      "charset" => {
        self.charset = Charset::try_from(value)?;
      }
      "collation" => {
        self.collation = Collation::try_from(value)?;
      }
      "enable_cleartext_plugin" => {
        self.enable_cleartext_plugin = true;
      }
      "no_engine_substitution" => {
        self.no_engine_substitution = true;
      }
      "pipes_as_concat" => {
        self.pipes_as_concat = true;
      }
      "set_names" => {
        self.set_names = true;
      }
      "timezone" => {
        self.timezone = Some(value);
      }
      _ => return Err(MysqlError::UnknownConfigurationParameter.into()),
    }
    Ok(())
  }
}
