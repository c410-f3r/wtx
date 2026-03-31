use crate::{
  crypto::{Agreement, CryptoError, P256AwsLcRs, P384AwsLcRs, X25519AwsLcRs},
  rng::CryptoRng,
};
use aws_lc_rs::{
  agreement::{
    ECDH_P256, ECDH_P384, EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519,
    agree_ephemeral,
  },
  rand::SystemRandom,
};

impl Agreement for P256AwsLcRs {
  type EphemeralSecretKey = EphemeralPrivateKey;
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 32];

  #[inline]
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(
      esk,
      UnparsedPublicKey::new(&ECDH_P256, other_participant_pk),
      CryptoError::DiffieHellmanError.into(),
      |value| {
        secret.copy_from_slice(value);
        crate::Result::Ok(())
      },
    )?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(&self, _: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(EphemeralPrivateKey::generate(&ECDH_P256, &SystemRandom::new())?)
  }

  #[inline]
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.compute_public_key()?)
  }
}

impl Agreement for P384AwsLcRs {
  type EphemeralSecretKey = EphemeralPrivateKey;
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 48];

  #[inline]
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(
      esk,
      UnparsedPublicKey::new(&ECDH_P384, other_participant_pk),
      CryptoError::DiffieHellmanError.into(),
      |value| {
        secret.copy_from_slice(value);
        crate::Result::Ok(())
      },
    )?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(&self, _: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(EphemeralPrivateKey::generate(&ECDH_P384, &SystemRandom::new())?)
  }

  #[inline]
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.compute_public_key()?)
  }
}

impl Agreement for X25519AwsLcRs {
  type EphemeralSecretKey = EphemeralPrivateKey;
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 32];

  #[inline]
  fn diffie_hellman(
    &self,
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(
      esk,
      UnparsedPublicKey::new(&X25519, other_participant_pk),
      CryptoError::DiffieHellmanError.into(),
      |value| {
        secret.copy_from_slice(value);
        crate::Result::Ok(())
      },
    )?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(&self, _: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(EphemeralPrivateKey::generate(&X25519, &SystemRandom::new())?)
  }

  #[inline]
  fn public_key(&self, esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.compute_public_key()?)
  }
}
