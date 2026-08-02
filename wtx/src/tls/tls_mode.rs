/// Indicates how streams should interpret TLS connections.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TlsMode {
  /// **NOT SECURE**
  ///
  /// Data is treated as plaintext bytes. Useful for tests or local connections.
  PlainText,
  /// Encrypted but **INSECURE**
  ///
  /// Data is encrypted but certificates are **NOT** verified.
  Unverified,
  /// Secure
  ///
  /// Data is encrypted and certificates are verified.
  #[default]
  Verified,
}

impl TlsMode {
  /// Returns `true` if this instance is [`TlsMode::PlainText`].
  #[inline]
  pub const fn is_plain_text(&self) -> bool {
    matches!(self, Self::PlainText)
  }

  /// Returns `true` if this instance is [`TlsMode::Unverified`].
  #[inline]
  pub const fn is_unverified(&self) -> bool {
    matches!(self, Self::Unverified)
  }

  /// Returns `true` if this instance is [`TlsMode::Verified`].
  #[inline]
  pub const fn is_verified(&self) -> bool {
    matches!(self, Self::Verified)
  }
}
