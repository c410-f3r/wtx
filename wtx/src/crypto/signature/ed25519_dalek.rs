use crate::crypto::{Ed25519RustCrypto, Signature};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};

impl Signature for Ed25519RustCrypto {
  type SignKey = SigningKey;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    let signature = sign_key.sign(msg);
    Ok(signature.to_bytes())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    VerifyingKey::try_from(pk)?.verify(msg, &ed25519_dalek::Signature::from_slice(signature)?)?;
    Ok(())
  }
}
