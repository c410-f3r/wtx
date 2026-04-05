use crate::crypto::{Ed25519Graviola, P256Graviola, P384Graviola, Signature};
use graviola::{
  hashing::{Sha256, Sha384},
  signing::{
    ecdsa::{P256, P384, SigningKey, VerifyingKey},
    eddsa::{Ed25519SigningKey, Ed25519VerifyingKey},
  },
};

impl Signature for P256Graviola {
  type SignKey = SigningKey<P256>;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    let mut buffer = [0; _];
    sign_key.sign::<Sha256>(&[msg], &mut buffer)?;
    Ok(buffer)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    VerifyingKey::<P256>::from_x962_uncompressed(pk)?.verify::<Sha256>(&[msg], signature)?;
    Ok(())
  }
}

impl Signature for P384Graviola {
  type SignKey = SigningKey<P384>;
  type SignOutput = [u8; 96];

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    let mut buffer = [0; _];
    sign_key.sign::<Sha384>(&[msg], &mut buffer)?;
    Ok(buffer)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    VerifyingKey::<P384>::from_x962_uncompressed(pk)?.verify::<Sha384>(&[msg], signature)?;
    Ok(())
  }
}

impl Signature for Ed25519Graviola {
  type SignKey = Ed25519SigningKey;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    Ok(sign_key.sign(msg))
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ed25519VerifyingKey::from_bytes(pk)?.verify(signature, msg)?;
    Ok(())
  }
}
