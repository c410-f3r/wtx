use crate::{
  crypto::{Agreement, CryptoError, P256Ring, P384Ring, X25519Ring},
  rng::CryptoRng,
};
use ring::{
  agreement::{
    ECDH_P256, ECDH_P384, EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519,
    agree_ephemeral,
  },
  rand::SystemRandom,
};

impl Agreement for P256Ring {
  type EphemeralSecretKey = EphemeralPrivateKey;
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 32];

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(esk, &UnparsedPublicKey::new(&ECDH_P256, other_participant_pk), |value| {
      secret.copy_from_slice(value);
    })
    .map_err(|_err| CryptoError::DiffieHellmanError)?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(
      EphemeralPrivateKey::generate(&ECDH_P256, &SystemRandom::new())
        .map_err(|_err| CryptoError::EphemeralSecretKeyError)?,
    )
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.compute_public_key().map_err(|_err| CryptoError::PublicKeyAgreementError)?)
  }
}

impl Agreement for P384Ring {
  type EphemeralSecretKey = EphemeralPrivateKey;
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 48];

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(esk, &UnparsedPublicKey::new(&ECDH_P384, other_participant_pk), |value| {
      secret.copy_from_slice(value);
    })
    .map_err(|_err| CryptoError::DiffieHellmanError)?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(
      EphemeralPrivateKey::generate(&ECDH_P384, &SystemRandom::new())
        .map_err(|_err| CryptoError::EphemeralSecretKeyError)?,
    )
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.compute_public_key().map_err(|_err| CryptoError::PublicKeyAgreementError)?)
  }
}

impl Agreement for X25519Ring {
  type EphemeralSecretKey = EphemeralPrivateKey;
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 32];

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(esk, &UnparsedPublicKey::new(&X25519, other_participant_pk), |value| {
      secret.copy_from_slice(value);
    })
    .map_err(|_err| CryptoError::DiffieHellmanError)?;
    Ok(secret)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(
      EphemeralPrivateKey::generate(&X25519, &SystemRandom::new())
        .map_err(|_err| CryptoError::EphemeralSecretKeyError)?,
    )
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.compute_public_key().map_err(|_err| CryptoError::PublicKeyAgreementError)?)
  }
}
