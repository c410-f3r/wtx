use crate::{
  database::client::mysql::{charset::Charset, collation::Collation},
  misc::UriRef,
};

/// Configuration
#[derive(Debug)]
pub struct Config<'data> {
  pub(crate) charset: Charset,
  pub(crate) collation: Collation,
  pub(crate) database: Option<&'data str>,
  pub(crate) host: &'data str,
  pub(crate) no_engine_substitution: bool,
  pub(crate) password: Option<&'data str>,
  pub(crate) pipes_as_concat: bool,
  pub(crate) port: u16,
  pub(crate) set_names: bool,
  pub(crate) timezone: &'data str,
  pub(crate) username: &'data str,
}

impl<'data> Config<'data> {
  /// Unwraps the elements from an URI.
  pub fn from_uri(uri: &'data UriRef<'_>) -> crate::Result<Config<'data>> {
    Ok(Self {
      charset: Charset::utf8mb4,
      collation: Charset::utf8mb4.default_collation(),
      database: None,
      host: "localhost",
      no_engine_substitution: true,
      password: None,
      pipes_as_concat: true,
      port: 3306,
      set_names: true,
      timezone: "+00:00",
      username: "root",
    })
  }
}
