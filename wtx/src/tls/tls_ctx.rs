pub(crate) mod hardened_sk_ctx;
pub(crate) mod plaintext_ctx;
pub(crate) mod sk_ctx;
pub(crate) mod trusted_ctx;
pub(crate) mod unverified_ctx;

use crate::{
  asn1::Asn1DecodeWrapperAux,
  codec::{Decode as _, DecodeWrapper, Pem},
  collections::{ShortBoxSliceU16, Vector},
  misc::SecretContext,
  rng::CryptoRng,
  tls::{SignatureScheme, TlsMode},
  x509::{KeyTy, Pkcs8},
};
use core::fmt::Debug;

/// TLS Context
///
/// Dictates how a TLS connection should behave.
pub trait TlsCtx: Debug {
  /// See [`TlsMode`].
  const TY: TlsMode;
}

/// TLS Context - Secret Keys
///
/// Dictates how a TLS connection should behave and also manages privates keys and theirs signatures.
pub trait TlsCtxSk: TlsCtx {
  /// Signature
  type Signature: AsRef<[u8]>;

  /// Sign the given message and return a digital signature.
  fn sign<RNG>(
    &self,
    buffer: &mut Vector<u8>,
    msg: &[u8],
    rng: &mut RNG,
    sc: SignatureScheme,
  ) -> crate::Result<Self::Signature>
  where
    RNG: CryptoRng;
}

/// Provides mechanisms to instantiate a secret key context from different formats.
pub trait TlsCtxSkLoader: Sized + TlsCtxSk {
  /// Input of the [`Self::from_ders`] method.
  type SkInputDer<'data>: TlsCtxSkInput<TlsCtxSk = Self>;
  /// Input of the [`Self::from_pems`] method.
  type SkInputPem<'data>: TlsCtxSkInput<TlsCtxSk = Self>;

  /// From a secret key in DER format.
  fn from_ders<'data, RNG>(
    input: impl IntoIterator<Item = Self::SkInputDer<'data>>,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng;

  /// From a secret key in PEM format.
  fn from_pems<'data, RNG>(
    input: impl IntoIterator<Item = Self::SkInputPem<'data>>,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng;
}

/// Used for type inference. Doesn't contain any internal logic.
pub trait TlsCtxSkInput {
  /// See [`TlsCtxSk`].
  type TlsCtxSk: TlsCtxSk;
}

impl TlsCtxSkInput for &[u8] {
  type TlsCtxSk = sk_ctx::SkCtx;
}

impl TlsCtxSkInput for ShortBoxSliceU16<u8> {
  type TlsCtxSk = sk_ctx::SkCtx;
}

impl TlsCtxSkInput for (SecretContext, &mut [u8]) {
  type TlsCtxSk = hardened_sk_ctx::HardenedSkCtx;
}

#[inline]
fn secret_key_from_pem(pem_bytes: &[u8]) -> crate::Result<(ShortBoxSliceU16<u8>, KeyTy)> {
  let mut buffer = Vector::new();
  let pem = Pem::<_, 1>::decode(&mut DecodeWrapper::new(pem_bytes, &mut buffer))?;
  let rslt = pem.data.into_inner()?;
  buffer.truncate(rslt[0].1.end);
  let key_ty = secret_key_ty(&buffer)?;
  Ok((buffer.try_into()?, key_ty))
}

#[inline]
fn secret_key_ty(der: &[u8]) -> crate::Result<KeyTy> {
  let mut dw = DecodeWrapper::new(der, Asn1DecodeWrapperAux::default());
  KeyTy::try_from(&Pkcs8::<&[u8]>::decode(&mut dw)?)
}
