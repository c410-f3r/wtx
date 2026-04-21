use crate::{
  collection::Vector,
  crypto::{
    Ed25519Graviola, Ed25519SignKeyGraviola, P256Graviola, P256SignKeyGraviola, P384Graviola,
    P384SignKeyGraviola, RsaPssRsaeSha256Graviola, RsaPssRsaeSha384Graviola,
    RsaPssSignKeySha256Graviola, RsaPssSignKeySha384Graviola, Signature,
  },
  rng::CryptoRng,
};
use graviola::{
  hashing::{Sha256, Sha384},
  signing::{
    ecdsa::{self, P256, P384},
    eddsa::Ed25519VerifyingKey,
    rsa,
  },
};

impl Signature for P256Graviola {
  type SignKey = P256SignKeyGraviola;
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
    let mut buffer = [0; _];
    let _ = sign_key.0.sign::<Sha256>(&[msg], &mut buffer)?;
    Ok(buffer)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    ecdsa::VerifyingKey::<P256>::from_x962_uncompressed(pk)?
      .verify_asn1::<Sha256>(&[msg], signature)?;
    Ok(())
  }
}

impl Signature for P384Graviola {
  type SignKey = P384SignKeyGraviola;
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
    let mut buffer = [0; _];
    let _ = sign_key.0.sign::<Sha384>(&[msg], &mut buffer)?;
    Ok(buffer)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    ecdsa::VerifyingKey::<P384>::from_x962_uncompressed(pk)?
      .verify_asn1::<Sha384>(&[msg], signature)?;
    Ok(())
  }
}

impl Signature for Ed25519Graviola {
  type SignKey = Ed25519SignKeyGraviola;
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
    Ok(sign_key.0.sign(msg))
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ed25519VerifyingKey::from_bytes(pk)?.verify(signature, msg)?;
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha256Graviola {
  type SignKey = RsaPssSignKeySha256Graviola;
  type SignOutput = Vector<u8>;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signature = Vector::from_vec(alloc::vec![0; sign_key.0.modulus_len_bytes()]);
    let _ = sign_key.0.sign_pss_sha256(&mut signature, msg)?;
    Ok(signature)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    rsa::VerifyingKey::from_pkcs1_der(pk)?.verify_pss_sha256(signature, msg)?;
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha384Graviola {
  type SignKey = RsaPssSignKeySha384Graviola;
  type SignOutput = Vector<u8>;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signature = Vector::from_vec(alloc::vec![0; sign_key.0.modulus_len_bytes()]);
    let _ = sign_key.0.sign_pss_sha384(&mut signature, msg)?;
    Ok(signature)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    rsa::VerifyingKey::from_pkcs1_der(pk)?.verify_pss_sha384(signature, msg)?;
    Ok(())
  }
}
