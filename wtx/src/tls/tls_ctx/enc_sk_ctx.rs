use crate::{
  collections::{ShortBoxSliceU8, Vector},
  crypto::DynSigningOutput,
  misc::{Secret, SecretContext, SensitiveBytes},
  rng::CryptoRng,
  tls::{
    SignatureScheme, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsError, TlsMode,
    tls_ctx::{secret_key_from_pem, secret_key_ty},
  },
  x509::KeyTy,
};
use core::hint::cold_path;

/// Encrypted Secret Key Context
///
/// Secure connection with protected private key. Data is encrypted and certificates are verified.
///
/// Used by servers.
#[derive(Debug, Default)]
pub struct EncSkCtx(ShortBoxSliceU8<(Secret, KeyTy)>);

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
    let kt = sc.cert_kt();
    for value in self.0.iter() {
      if value.1 == kt {
        return value.0.peek(&mut buffer.into(), |sp| {
          sc.handshake_st().sign_key_from_pkcs8(sp.data())?.sign(msg, rng)
        })?;
      }
    }
    cold_path();
    Err(TlsError::UnsupportedSignAlgorithm.into())
  }
}

impl TlsCtxSkLoader for EncSkCtx {
  type SkInputDer<'data> = (SecretContext, &'data mut [u8]);
  type SkInputPem<'data> = (SecretContext, &'data mut [u8]);

  #[inline]
  fn from_ders<'data, RNG>(
    input: impl IntoIterator<Item = Self::SkInputDer<'data>>,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut vector = Vector::new();
    for (secret_context, secret_key) in input {
      let key_ty = secret_key_ty(secret_key)?;
      vector.push((Secret::new(secret_key, rng, secret_context)?, key_ty))?;
    }
    Ok(Self(vector.try_into()?))
  }

  /// From a secret key in PEM format.
  #[inline]
  fn from_pems<'data, RNG>(
    input: impl IntoIterator<Item = Self::SkInputPem<'data>>,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut vector = Vector::new();
    for (secret_context, pem_bytes) in input {
      let sb = SensitiveBytes::new(pem_bytes);
      let (mut secret_key, key_ty) = secret_key_from_pem(*sb)?;
      vector.push((Secret::new(&mut secret_key, rng, secret_context)?, key_ty))?;
    }
    Ok(Self(vector.try_into()?))
  }
}
