use crate::database::client::mysql::{charset::Charset, collation::Collation};

pub struct Config<'data> {
  pub(crate) charset: Charset,
  pub(crate) collation: Option<Collation>,
  pub(crate) database: Option<&'data str>,
  pub(crate) host: &'data str,
  pub(crate) no_engine_substitution: bool,
  pub(crate) password: Option<&'data str>,
  pub(crate) pipes_as_concat: bool,
  pub(crate) port: u16,
  pub(crate) set_names: bool,
  pub(crate) statement_cache_capacity: usize,
  pub(crate) timezone: Option<&'data str>,
  pub(crate) username: &'data str,
}
