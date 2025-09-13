/// Indicates how `TlsStream` should interpret connections.
pub trait TlsMode {
  /// See [`TlsModeTy`].
  const TY: TlsModeTy;
}

/// Interprets connections as plaintext bytes.
pub struct TlsDisable;
impl TlsMode for TlsDisable {
  const TY: TlsModeTy = TlsModeTy::Disable;
}

/// TLS connections are strictly enforced.
pub struct TlsVerifyFull;
impl TlsMode for TlsVerifyFull {
  const TY: TlsModeTy = TlsModeTy::VerifyFull;
}

/// Indicates how streams should interpret TLS connections.
pub enum TlsModeTy {
  /// Connections are treated as plaintext bytes.
  Disable,
  /// TLS connections are strictly enforced.
  VerifyFull,
}
