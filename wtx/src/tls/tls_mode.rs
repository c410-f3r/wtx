/// Indicates how `TlsStream` should interpret connections.
pub trait TlsMode {
  /// See [`TlsModeTy`].
  const TY: TlsModeTy;
}

/// By-passes TLS machinery. Useful for tests or local connections.
#[derive(Debug)]
pub struct TlsModePlainText;
impl TlsMode for TlsModePlainText {
  const TY: TlsModeTy = TlsModeTy::PlainText;
}

/// TLS connections are strictly enforced.
#[derive(Debug)]
pub struct TlsModeVerifyFull;
impl TlsMode for TlsModeVerifyFull {
  const TY: TlsModeTy = TlsModeTy::VerifyFull;
}

/// Indicates how streams should interpret TLS connections.
#[derive(Debug, Eq, PartialEq)]
pub enum TlsModeTy {
  /// Connections are treated as plaintext bytes. Useful for tests or local connections.
  PlainText,
  /// TLS connections are strictly enforced.
  VerifyFull,
}

impl TlsModeTy {
  #[must_use]
  pub(crate) const fn is_plain_text(&self) -> bool {
    matches!(self, Self::PlainText)
  }
}
