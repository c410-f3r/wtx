use crate::{
  crypto::{P384RustCrypto, P384SignKeyRustCrypto, Signature},
  rng::CryptoRng,
};
use p384::ecdsa::{VerifyingKey, signature::Signer};
use signature::Verifier as _;

impl Signature for P384RustCrypto {
  type SignKey = P384SignKeyRustCrypto;
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
    VerifyingKey::from_sec1_bytes(pk)?
      .verify(msg, &p384::ecdsa::Signature::from_der(signature)?)?;
    Ok(())
  }
}
