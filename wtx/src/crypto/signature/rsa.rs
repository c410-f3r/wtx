use crate::{
  crypto::{
    RsaPssRsaeSha256RustCrypto, RsaPssRsaeSha384RustCrypto, RsaPssSignKeySha256RustCrypto,
    RsaPssSignKeySha384RustCrypto, Signature,
  },
  rng::CryptoRng,
};
use alloc::boxed::Box;
use rsa::{RsaPublicKey, pss::VerifyingKey, signature::Verifier as _};
use sha2::{Sha256, Sha384};
use signature::RandomizedSigner;
use spki::DecodePublicKey;

impl Signature for RsaPssRsaeSha256RustCrypto {
  type SignKey = RsaPssSignKeySha256RustCrypto;
  type SignOutput = Box<[u8]>;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let sign = sign_key.0.sign_with_rng(rng, msg);
    let inner: Box<[u8]> = sign.into();
    Ok(inner)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let pk = RsaPublicKey::from_public_key_der(pk)?;
    let verifying_key = VerifyingKey::<Sha256>::new(pk);
    let signature = rsa::pss::Signature::try_from(signature)?;
    verifying_key.verify(msg, &signature)?;
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha384RustCrypto {
  type SignKey = RsaPssSignKeySha384RustCrypto;
  type SignOutput = Box<[u8]>;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let sign = sign_key.0.sign_with_rng(rng, msg);
    let inner: Box<[u8]> = sign.into();
    Ok(inner)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let pk = RsaPublicKey::from_public_key_der(pk)?;
    let verifying_key = VerifyingKey::<Sha384>::new(pk);
    let signature = rsa::pss::Signature::try_from(signature)?;
    verifying_key.verify(msg, &signature)?;
    Ok(())
  }
}
