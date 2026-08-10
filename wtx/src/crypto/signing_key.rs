#[cfg(feature = "crypto-alr")]
mod alr;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-ring")]
mod ring;
#[cfg(feature = "crypto-ruco")]
mod ruco;

use crate::{
  crypto::{HashTy, SigningOutput},
  misc::DefaultArray,
  rng::CryptoRng,
};
use core::marker::PhantomData;

/// A cryptographic secret usually composed by a secret key and a public key.
///
/// `hash_ty` is only used by RSA algorithms. For anything else such a field can be ignored.
pub trait SigningKey: Sized {
  /// Output of the [`Self::sign`] method.
  type Signature: AsRef<[u8]>;

  /// New instance from a private key.
  ///
  /// `hash_ty` is only used by RSA algorithms. For anything else such a field can be ignored.
  fn from_pkcs8(bytes: &[u8], hash_ty: HashTy) -> crate::Result<Self>;

  /// Sign the given message and return a digital signature.
  fn sign<RNG>(&self, msg: &[u8], rng: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng;

  /// Checks if the `signature` derived from `msg` was signed by `pubkey` .
  fn validate(msg: &[u8], pk: &[u8], so: &SigningOutput<&[u8]>) -> crate::Result<()>;
}

/// Dummy [`SigningKeyDummy`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SigningKeyDummy<S>(PhantomData<S>);

impl<S> SigningKey for SigningKeyDummy<S>
where
  S: AsRef<[u8]> + DefaultArray,
{
  type Signature = S;

  #[inline]
  fn from_pkcs8(_: &[u8], _: HashTy) -> crate::Result<Self> {
    Ok(Self(PhantomData))
  }

  #[inline]
  fn sign<RNG>(&self, _: &[u8], _: &mut RNG) -> crate::Result<SigningOutput<Self::Signature>>
  where
    RNG: CryptoRng,
  {
    Ok(SigningOutput::new(HashTy::Sha256, S::default_array()))
  }

  #[inline]
  fn validate(_: &[u8], _: &[u8], _: &SigningOutput<&[u8]>) -> crate::Result<()> {
    Ok(())
  }
}
