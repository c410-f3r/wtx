use crate::{
  collections::{ShortBoxSliceU8, ShortBoxSliceU16, Vector},
  crypto::DynSigningOutput,
  rng::CryptoRng,
  tls::{
    SignatureScheme, TlsCtx, TlsCtxSk, TlsCtxSkLoader, TlsError, TlsMode,
    tls_ctx::{secret_key_from_pem, secret_key_ty},
  },
  x509::KeyTy,
};
use core::hint::cold_path;

/// Secret Key Context
///
/// Secure connection with unprotected private key. Data is encrypted and certificates are verified.
///
/// Used by servers.
#[derive(Debug, Default)]
pub struct SkCtx(ShortBoxSliceU8<(ShortBoxSliceU16<u8>, KeyTy)>);

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
    let kt = sc.cert_kt();
    for value in self.0.iter() {
      if value.1 == kt {
        return sc.handshake_st().sign_key_from_pkcs8(&value.0)?.sign(msg, rng);
      }
    }
    cold_path();
    Err(TlsError::UnsupportedSignAlgorithm.into())
  }
}

impl TlsCtxSkLoader for SkCtx {
  type SkInputDer<'data> = ShortBoxSliceU16<u8>;
  type SkInputPem<'data> = &'data [u8];

  #[inline]
  fn from_ders<'data, RNG>(
    input: impl IntoIterator<Item = Self::SkInputDer<'data>>,
    _: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut vector = Vector::new();
    for value in input {
      let key_ty = secret_key_ty(&value)?;
      vector.push((value, key_ty))?;
    }
    Ok(Self(vector.try_into()?))
  }

  /// From a secret key in PEM format.
  #[inline]
  fn from_pems<'data, RNG>(
    input: impl IntoIterator<Item = Self::SkInputPem<'data>>,
    _: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut vector = Vector::new();
    for pem in input {
      vector.push(secret_key_from_pem(pem)?)?;
    }
    Ok(Self(vector.try_into()?))
  }
}
