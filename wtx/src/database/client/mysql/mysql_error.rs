/// Error
#[derive(Debug)]
pub enum MysqlError {
  /// Invalid auth plugin bytes
  InvalidAuthPluginBytes,
  /// Invalid auth switch bytes
  InvalidAuthSwitchBytes,
  /// Invalid binary row bytes
  InvalidBinaryRowBytes,
  /// Invalid column bytes
  InvalidColumnBytes,
  /// Invalid connection bytes
  InvalidConnectionBytes,
  /// Invalid end of file bytes
  InvalidEofBytes,
  /// Invalid fetch bytes
  InvalidFetchBytes,
  /// Invalid handshake bytes
  InvalidHandshakeBytes,
  /// Invalid lenenc content bytes
  InvalidLenencBytes,
  /// Invalid lenenc bytes
  InvalidLenencContentBytes,
  /// Invalid OK bytes
  InvalidOkBytes,
  /// Invalid prepare bytes
  InvalidPrepareBytes,
  /// Invalid text row bytes
  InvalidTextRowBytes,
  /// Fetch command expected one result but got zero or more than one results
  NonSingleFetch,
  /// Unknown authentication method
  UnknownAuthPlugin,
  /// Unknown configuration parameter
  UnknownConfigurationParameter,
  /// Mysql server does not support SSL
  UnsupportedServerSsl,
}
