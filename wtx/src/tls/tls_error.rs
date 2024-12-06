/// TLS errror
#[derive(Debug)]
pub enum TlsError {
  /// It is necessary to provide a crypto provide using Cargo features
  MissingProvider,
}
