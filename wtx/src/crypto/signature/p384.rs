use crate::crypto::{P384RustCrypto, Signature};
use p384::ecdsa::{SigningKey, VerifyingKey, signature::Signer};
use signature::Verifier as _;

impl Signature for P384RustCrypto {
  type SignKey = SigningKey;
  type SignOutput = [u8; 96];

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    let signature: p384::ecdsa::Signature = sign_key.sign(msg);
    Ok(signature.to_bytes().into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    VerifyingKey::from_sec1_bytes(pk)?
      .verify(msg, &p384::ecdsa::Signature::from_bytes(signature.try_into()?)?)?;
    Ok(())
  }
}
