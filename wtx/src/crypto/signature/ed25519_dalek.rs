use crate::{
  crypto::{Ed25519RustCrypto, Ed25519SignKeyRustCrypto, Signature},
  rng::CryptoRng,
};
use ed25519_dalek::{Signer, Verifier, VerifyingKey};

impl Signature for Ed25519RustCrypto {
  type SignKey = Ed25519SignKeyRustCrypto;
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
    VerifyingKey::from_bytes(pk.try_into()?)?
      .verify(msg, &ed25519_dalek::Signature::from_slice(signature)?)?;
    Ok(())
  }
}
