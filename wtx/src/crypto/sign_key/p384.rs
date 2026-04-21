use crate::{
  crypto::{P384SignKeyRustCrypto, sign_key::SignKey},
  rng::CryptoRng,
};
use p384::ecdsa::SigningKey;
use pkcs8::DecodePrivateKey;

impl SignKey for P384SignKeyRustCrypto {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(SigningKey::from_pkcs8_der(bytes)?))
  }

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut secret = [0u8; 48];
    rng.fill_slice(&mut secret);
    Self::from_pkcs8(&secret)
  }
}
