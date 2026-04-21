use crate::{crypto::SignKey, misc::DefaultArray, rng::CryptoRng};
use core::marker::PhantomData;

#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "x25519-dalek")]
mod ed25519_dalek;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "p256")]
mod p256;
#[cfg(feature = "p384")]
mod p384;
#[cfg(feature = "crypto-ring")]
mod ring;
#[cfg(feature = "rsa")]
mod rsa;
pub(crate) mod signature_ty;

/// A mathematical scheme for verifying the authenticity of digital messages or documents.
pub trait Signature {
  /// The structure used to sign messages
  type SignKey: SignKey;
  /// The result of a signing operation
  type SignOutput;

  /// Checks if the `signature` derived from `msg` was signed by `pubkey` .
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng;

  /// Checks if the `signature` derived from `msg` was signed by `pubkey` .
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()>;
}

/// Stub [`Signature`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SignatureStub<SK, SO>(PhantomData<(SK, SO)>);

impl<SK, SO> Signature for SignatureStub<SK, SO>
where
  SK: SignKey,
  SO: DefaultArray,
{
  type SignKey = SK;
  type SignOutput = SO;

  #[inline]
  fn sign<RNG>(_: &mut RNG, _: &mut Self::SignKey, _: &[u8]) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    Ok(Self::SignOutput::default_array())
  }

  #[inline]
  fn validate(_: &[u8], _: &[u8], _: &[u8]) -> crate::Result<()> {
    Ok(())
  }
}
