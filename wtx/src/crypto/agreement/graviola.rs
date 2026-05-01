use crate::{
  crypto::{Agreement, AsRefWrapper, CryptoError, P256Graviola, P384Graviola, X25519Graviola},
  rng::CryptoRng,
};
use graviola::key_agreement::{p256, p384, x25519};

impl Agreement for P256Graviola {
  type EphemeralSecretKey = p256::PrivateKey;
  type PublicKey = [u8; 65];
  type SharedSecret = AsRefWrapper<p256::SharedSecret>;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let pk_bytes: &[u8; 65] =
      other_participant_pk.try_into().map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    let pk = p256::PublicKey::from_x962_uncompressed(pk_bytes)
      .map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    let shared = esk.diffie_hellman(&pk).map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    Ok(AsRefWrapper(shared))
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(p256::PrivateKey::new_random()?)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.public_key_uncompressed())
  }
}

impl Agreement for P384Graviola {
  type EphemeralSecretKey = p384::PrivateKey;
  type PublicKey = [u8; 97];
  type SharedSecret = AsRefWrapper<p384::SharedSecret>;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let pk_bytes: &[u8; 97] =
      other_participant_pk.try_into().map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    let pk = p384::PublicKey::from_x962_uncompressed(pk_bytes)
      .map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    let shared = esk.diffie_hellman(&pk).map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    Ok(AsRefWrapper(shared))
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(p384::PrivateKey::new_random()?)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.public_key_uncompressed())
  }
}

impl Agreement for X25519Graviola {
  type EphemeralSecretKey = x25519::PrivateKey;
  type PublicKey = [u8; 32];
  type SharedSecret = AsRefWrapper<x25519::SharedSecret>;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    let pk_bytes: &[u8; 32] =
      other_participant_pk.try_into().map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    let pk = x25519::PublicKey::from_array(pk_bytes);
    let shared = esk.diffie_hellman(&pk).map_err(|_err| CryptoError::PublicKeyAgreementError)?;
    Ok(AsRefWrapper(shared))
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(_: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    Ok(x25519::PrivateKey::new_random()?)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    Ok(esk.public_key().as_bytes())
  }
}

impl AsRef<[u8]> for AsRefWrapper<p256::SharedSecret> {
  fn as_ref(&self) -> &[u8] {
    &self.0.0
  }
}

impl AsRef<[u8]> for AsRefWrapper<p384::SharedSecret> {
  fn as_ref(&self) -> &[u8] {
    &self.0.0
  }
}

impl AsRef<[u8]> for AsRefWrapper<x25519::SharedSecret> {
  fn as_ref(&self) -> &[u8] {
    &self.0.0
  }
}
