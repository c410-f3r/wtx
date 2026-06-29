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
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 32];

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(
      EphemeralPrivateKey::generate(&ECDH_P256, &SystemRandom::new())
        .map_err(|_err| CryptoError::PublicKeyAgreementError)?,
    ))
  }

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(
      self.0,
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
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    Ok(self.0.compute_public_key().map_err(|_err| CryptoError::PublicKeyAgreementError)?)
  }
}

impl Agreement for P384AwsLcRs {
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 48];

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(
      EphemeralPrivateKey::generate(&ECDH_P384, &SystemRandom::new())
        .map_err(|_err| CryptoError::PublicKeyAgreementError)?,
    ))
  }

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(
      self.0,
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
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    Ok(self.0.compute_public_key().map_err(|_err| CryptoError::PublicKeyAgreementError)?)
  }
}

impl Agreement for X25519AwsLcRs {
  type PublicKey = PublicKey;
  type SharedSecret = [u8; 32];

  #[inline]
  fn generate<RNG>(_: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(
      EphemeralPrivateKey::generate(&X25519, &SystemRandom::new())
        .map_err(|_err| CryptoError::PublicKeyAgreementError)?,
    ))
  }

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    let mut secret = [0u8; _];
    agree_ephemeral(
      self.0,
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
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    Ok(self.0.compute_public_key().map_err(|_err| CryptoError::PublicKeyAgreementError)?)
  }
}
