use crate::{
  crypto::{
    Ed25519Ruco, Ed25519SignKeyRuco, P256Ruco, P256SignKeyRuco, P384Ruco, P384SignKeyRuco,
    RsaPssRsaeSha256Ruco, RsaPssRsaeSha384Ruco, RsaPssSignKeySha256Ruco, RsaPssSignKeySha384Ruco,
    Signature,
  },
  rng::CryptoRng,
};
use alloc::boxed::Box;
use signature::{RandomizedSigner as _, Signer as _, Verifier as _};
use spki::DecodePublicKey as _;

impl Signature for Ed25519Ruco {
  type SignKey = Ed25519SignKeyRuco;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let signature = sign_key.0.try_sign(msg)?;
    Ok(signature.to_bytes())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    ed25519_dalek::VerifyingKey::from_bytes(pk.try_into()?)?
      .verify(msg, &ed25519_dalek::Signature::from_slice(signature)?)?;
    Ok(())
  }
}

impl Signature for P256Ruco {
  type SignKey = P256SignKeyRuco;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let signature: p256::ecdsa::Signature = sign_key.0.sign(msg);
    Ok(signature.to_bytes().into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    p256::ecdsa::VerifyingKey::from_sec1_bytes(pk)?
      .verify(msg, &p256::ecdsa::Signature::from_der(signature)?)?;
    Ok(())
  }
}

impl Signature for P384Ruco {
  type SignKey = P384SignKeyRuco;
  type SignOutput = [u8; 96];

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let signature: p384::ecdsa::Signature = sign_key.0.sign(msg);
    Ok(signature.to_bytes().into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    p384::ecdsa::VerifyingKey::from_sec1_bytes(pk)?
      .verify(msg, &p384::ecdsa::Signature::from_der(signature)?)?;
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha256Ruco {
  type SignKey = RsaPssSignKeySha256Ruco;
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
    let rpk = rsa::RsaPublicKey::from_public_key_der(pk)?;
    let verifying_key = rsa::pss::VerifyingKey::<sha2::Sha256>::new(rpk);
    let sig = rsa::pss::Signature::try_from(signature)?;
    verifying_key.verify(msg, &sig)?;
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha384Ruco {
  type SignKey = RsaPssSignKeySha384Ruco;
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
    let rpk = rsa::RsaPublicKey::from_public_key_der(pk)?;
    let verifying_key = rsa::pss::VerifyingKey::<sha2::Sha384>::new(rpk);
    let sig = rsa::pss::Signature::try_from(signature)?;
    verifying_key.verify(msg, &sig)?;
    Ok(())
  }
}
