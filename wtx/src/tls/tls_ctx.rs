pub(crate) mod enc_sk_ctx;
pub(crate) mod plaintext_ctx;
pub(crate) mod sk_ctx;
pub(crate) mod trusted_ctx;
pub(crate) mod unverified_ctx;

use crate::{
  asn1::{Asn1DecodeWrapperAux, Pkcs8},
  codec::{Decode as _, DecodeWrapper, Pem},
  collections::Vector,
  misc::SecretContext,
  rng::CryptoRng,
  tls::{SignatureScheme, TlsMode},
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
  /// Input of the [`Self::from_der`] method.
  type SkInputDer<'data>: TlsCtxSkInput<TlsCtxSk = Self>;
  /// Input of the [`Self::from_pem`] method.
  type SkInputPem<'data>: TlsCtxSkInput<TlsCtxSk = Self>;

  /// From a secret key in DER format.
  fn from_der<RNG>(input: Self::SkInputDer<'_>, rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng;

  /// From a secret key in PEM format.
  fn from_pem<RNG>(input: Self::SkInputPem<'_>, rng: &mut RNG) -> crate::Result<Self>
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

impl TlsCtxSkInput for Vector<u8> {
  type TlsCtxSk = sk_ctx::SkCtx;
}

impl TlsCtxSkInput for (SecretContext, &mut [u8]) {
  type TlsCtxSk = enc_sk_ctx::EncSkCtx;
}

#[inline]
fn secret_key_from_pem(input: &[u8]) -> Result<Vector<u8>, crate::Error> {
  let mut buffer = Vector::new();
  let pem = Pem::<_, 1>::decode(&mut DecodeWrapper::new(input, &mut buffer))?;
  let rslt = pem.data.into_inner()?;
  buffer.truncate(rslt[0].1.end);
  let mut dw = DecodeWrapper::new(&buffer, Asn1DecodeWrapperAux::default());
  let _pkcs8 = Pkcs8::<&[u8]>::decode(&mut dw)?;
  Ok(buffer)
}
