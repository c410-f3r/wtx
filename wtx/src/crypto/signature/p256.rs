use crate::{
  crypto::{P256RustCrypto, P256SignKeyRustCrypto, Signature},
  rng::CryptoRng,
};
use p256::ecdsa::{VerifyingKey, signature::Signer};
use signature::Verifier as _;

impl Signature for P256RustCrypto {
  type SignKey = P256SignKeyRustCrypto;
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
    VerifyingKey::from_sec1_bytes(pk)?
      .verify(msg, &p256::ecdsa::Signature::from_der(signature)?)?;
    Ok(())
  }
}
