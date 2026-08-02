use crate::{
  collections::Vector,
  rng::CryptoRng,
  tls::{SignatureScheme, TlsCtx, TlsCtxSk, TlsMode},
};

/// Plaintext Context
///
/// **INSECURE**
///
/// There are no TLS handshakes or certificate validations and data is treated as plaintext
/// bytes. Useful for tests or local connections.
///
/// Can be used by clients or servers.
#[derive(Clone, Debug, Default)]
pub struct PlaintextCtx {}

impl PlaintextCtx {
  /// New instance
  #[inline]
  pub const fn new() -> Self {
    Self {}
  }
}

impl TlsCtx for PlaintextCtx {
  const TY: TlsMode = TlsMode::PlainText;
}

impl TlsCtxSk for PlaintextCtx {
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
