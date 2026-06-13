/// Indicates how TLS streams should interpret connections.
pub trait TlsMode: Default {
  /// See [`TlsModeTy`].
  const TY: TlsModeTy;
}

/// **NOT SECURE**
///
/// Data is treated as plaintext bytes. Useful for tests or local connections.
#[derive(Clone, Debug, Default)]
pub struct TlsModePlainText;
impl TlsMode for TlsModePlainText {
  const TY: TlsModeTy = TlsModeTy::PlainText;
}

/// Encrypted but **INSECURE**
///
/// Data is encrypted but certificates are **NOT** verified.
#[derive(Clone, Debug, Default)]
pub struct TlsModeUnverified;
impl TlsMode for TlsModeUnverified {
  const TY: TlsModeTy = TlsModeTy::Unverified;
}

/// Secure
///
/// Data is encrypted and certificates are verified.
#[derive(Clone, Debug, Default)]
pub struct TlsModeVerified;
impl TlsMode for TlsModeVerified {
  const TY: TlsModeTy = TlsModeTy::Verified;
}

/// Strictly secure
///
/// Data is encrypted and certificates are heavily inspected. This version differs from
/// [`TlsModeVerified`] in that some valid but lenient certificates might fail.
#[derive(Clone, Debug, Default)]
pub struct TlsModeStrict;
impl TlsMode for TlsModeStrict {
  const TY: TlsModeTy = TlsModeTy::Strict;
}

/// Indicates how streams should interpret TLS connections.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TlsModeTy {
  /// See [`TlsModePlainText`].
  PlainText,
  /// See [`TlsModeUnverified`].
  Unverified,
  /// See [`TlsModeVerified`].
  #[default]
  Verified,
  /// See [`TlsModeStrict`].
  Strict,
}

impl TlsModeTy {
  /// Returns `true` if this instance is [`TlsModeTy::PlainText`].
  #[inline]
  pub const fn is_plain_text(&self) -> bool {
    matches!(self, Self::PlainText)
  }

  /// Everything but [`TlsModeTy::PlainText`] and [`TlsModeTy::Unverified`] requires a one or more
  /// certifications for evaluations purposes.
  #[inline]
  pub const fn require_certs(&self) -> bool {
    matches!(self, Self::Verified | Self::Strict)
  }
}
