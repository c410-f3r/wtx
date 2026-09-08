use crate::{
  collections::{ShortBoxSliceU8, Vector},
  crypto::DynSigningOutput,
  rng::CryptoRng,
  secret::{Secret, SecretSlice, SecretStr},
  tls::{
    SignatureScheme, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsError, TlsMode,
    tls_ctx::{secret_key_from_pem, secret_key_ty},
  },
  x509::KeyTy,
};
use core::hint::cold_path;

/// Hardened Secret Key Context
///
/// Secure connection with private keys protected using the [`Secret`] structure. Data is encrypted
/// and certificates are verified.
///
/// Used by servers.
#[derive(Debug, Default)]
pub struct HardenedSkCtx(ShortBoxSliceU8<(SecretSlice, KeyTy)>);

impl TlsCtx for HardenedSkCtx {
  const TY: TlsMode = TlsMode::Verified;
}

impl TlsCtxSk for HardenedSkCtx {
  type Signature = DynSigningOutput;

  #[inline]
  fn sign<RNG>(
    &self,
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
        return sc.handshake_st().sign_key_from_pkcs8(&value.0.peek()?)?.sign(msg, rng);
      }
    }
    cold_path();
    Err(TlsError::UnsupportedSignAlgorithm.into())
  }
}

impl TlsCtxSkLoader for HardenedSkCtx {
  type SkInputDer<'data> = SecretSlice;
  type SkInputPem<'data> = SecretStr;

  #[inline]
  fn from_ders<'data>(
    input: impl IntoIterator<Item = Self::SkInputDer<'data>>,
  ) -> crate::Result<Self> {
    let mut vector = Vector::new();
    for secret_key in input {
      let key_ty = secret_key_ty(&secret_key.peek()?)?;
      vector.push((secret_key, key_ty))?;
    }
    Ok(Self(vector.try_into()?))
  }

  /// From a secret key in PEM format.
  #[inline]
  fn from_pems<'data>(
    input: impl IntoIterator<Item = Self::SkInputPem<'data>>,
  ) -> crate::Result<Self> {
    let mut vector = Vector::new();
    for pem in input {
      let (mut secret_key, key_ty) = secret_key_from_pem(pem.peek()?.as_bytes())?;
      vector.push((Secret::new(&mut *secret_key)?, key_ty))?;
    }
    Ok(Self(vector.try_into()?))
  }
}
