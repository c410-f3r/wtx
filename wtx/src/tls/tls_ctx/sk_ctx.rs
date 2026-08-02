use crate::{
  collections::Vector,
  crypto::DynSigningOutput,
  rng::CryptoRng,
  tls::{SignatureScheme, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsMode, tls_ctx::secret_key_from_pem},
};

/// Secret Key Context
///
/// Secure connection with unprotected private key. Data is encrypted and certificates are verified.
///
/// Used by servers.
#[derive(Debug, Default)]
pub struct SkCtx(Vector<u8>);

impl TlsCtx for SkCtx {
  const TY: TlsMode = TlsMode::Verified;
}

impl TlsCtxSk for SkCtx {
  type Signature = DynSigningOutput;

  #[inline]
  fn sign<RNG>(
    &self,
    _: &mut Vector<u8>,
    msg: &[u8],
    rng: &mut RNG,
    sc: SignatureScheme,
  ) -> crate::Result<Self::Signature>
  where
    RNG: CryptoRng,
  {
    sc.handshake_st().sign_key_from_pkcs8(&self.0)?.sign(msg, rng)
  }
}

impl TlsCtxSkLoader for SkCtx {
  type SkInputDer<'data> = Vector<u8>;
  type SkInputPem<'data> = &'data [u8];

  #[inline]
  fn from_der<RNG>(input: Self::SkInputDer<'_>, _: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(input))
  }

  /// From a secret key in PEM format.
  #[inline]
  fn from_pem<RNG>(input: Self::SkInputPem<'_>, _: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(secret_key_from_pem(input)?))
  }
}
