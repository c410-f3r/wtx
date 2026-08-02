use crate::{
  collections::Vector,
  crypto::DynSigningOutput,
  misc::{Secret, SecretContext, SensitiveBytes},
  rng::CryptoRng,
  tls::{SignatureScheme, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsMode, tls_ctx::secret_key_from_pem},
};

/// Encrypted Secret Key Context
///
/// Secure connection with protected private key. Data is encrypted and certificates are verified.
///
/// Used by servers.
#[derive(Debug, Default)]
pub struct EncSkCtx(Secret);

impl TlsCtx for EncSkCtx {
  const TY: TlsMode = TlsMode::Verified;
}

impl TlsCtxSk for EncSkCtx {
  type Signature = DynSigningOutput;

  #[inline]
  fn sign<RNG>(
    &self,
    buffer: &mut Vector<u8>,
    msg: &[u8],
    rng: &mut RNG,
    sc: SignatureScheme,
  ) -> crate::Result<Self::Signature>
  where
    RNG: CryptoRng,
  {
    self.0.peek(&mut buffer.into(), |sp| {
      sc.handshake_st().sign_key_from_pkcs8(sp.data())?.sign(msg, rng)
    })?
  }
}

impl TlsCtxSkLoader for EncSkCtx {
  type SkInputDer<'data> = (SecretContext, &'data mut [u8]);
  type SkInputPem<'data> = (SecretContext, &'data mut [u8]);

  #[inline]
  fn from_der<RNG>(
    (secret_context, secret_key): Self::SkInputDer<'_>,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(Secret::new(secret_key, rng, secret_context)?))
  }

  /// From a secret key in PEM format.
  #[inline]
  fn from_pem<RNG>(
    (secret_context, secret_key): Self::SkInputPem<'_>,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let sb = SensitiveBytes::new(secret_key);
    let mut bytes = secret_key_from_pem(*sb)?;
    Ok(Self(Secret::new(&mut bytes, rng, secret_context)?))
  }
}
