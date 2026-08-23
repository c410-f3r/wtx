use crate::tls::{TlsCtx, TlsMode};

/// Trusted Context
///
/// Secure connection for clients that rely on trusted anchors. Data is encrypted and certificates
/// are verified.
///
/// Used by clients without mTLS.
#[derive(Debug, Default)]
pub struct TrustedCtx {}

impl TrustedCtx {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }
}

impl TlsCtx for TrustedCtx {
  const TY: TlsMode = TlsMode::Verified;
}
