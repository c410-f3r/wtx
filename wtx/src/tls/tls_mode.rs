/// Indicates how `TlsStream` should interpret connections.
pub trait TlsMode {
  /// See [`TlsModeTy`].
  const TY: TlsModeTy;
}

/// By-passes TLS machinery. Useful for tests or local connections.
pub struct TlsModePlainText;
impl TlsMode for TlsModePlainText {
  const TY: TlsModeTy = TlsModeTy::PlainText;
}

/// TLS connections are strictly enforced.
pub struct TlsModeVerifyFull;
impl TlsMode for TlsModeVerifyFull {
  const TY: TlsModeTy = TlsModeTy::VerifyFull;
}

/// Indicates how streams should interpret TLS connections.
pub enum TlsModeTy {
  /// Connections are treated as plaintext bytes. Useful for tests or local connections.
  PlainText,
  /// TLS connections are strictly enforced.
  VerifyFull,
}
