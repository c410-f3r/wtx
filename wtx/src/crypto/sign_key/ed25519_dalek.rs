use crate::{
  crypto::{Ed25519SignKeyRustCrypto, sign_key::SignKey},
  rng::CryptoRng,
};
use ed25519_dalek::SigningKey;
use pkcs8::DecodePrivateKey;

impl SignKey for Ed25519SignKeyRustCrypto {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut secret = [0u8; 32];
    rng.fill_slice(&mut secret);
    Self::from_pkcs8(&secret)
  }
}
