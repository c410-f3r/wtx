/// Error
#[derive(Debug)]
pub enum MysqlError {
  /// Unknown authentication method
  UnknownAuthPlugin,
  /// Unknown configuration parameter
  UnknownConfigurationParameter,
}
