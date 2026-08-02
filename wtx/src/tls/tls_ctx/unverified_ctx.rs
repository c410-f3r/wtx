use crate::{
  collections::Vector,
  rng::CryptoRng,
  tls::{SignatureScheme, TlsCtx, TlsCtxSk, TlsMode},
};

/// Unverified Context
///
/// **INSECURE**
///
/// TLS handshakes are performed and data is encrypted but certificates are **NOT** verified.
///
/// Can be used by clients or servers.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnverifiedCtx {}

impl UnverifiedCtx {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }
}

impl TlsCtx for UnverifiedCtx {
  const TY: TlsMode = TlsMode::Unverified;
}

impl TlsCtxSk for UnverifiedCtx {
  type Signature = [u8; 0];

  #[inline]
  fn sign<RNG>(
    &self,
    _: &mut Vector<u8>,
    _: &[u8],
    _: &mut RNG,
    _: SignatureScheme,
  ) -> crate::Result<Self::Signature>
  where
    RNG: CryptoRng,
  {
    Ok([])
  }
}
