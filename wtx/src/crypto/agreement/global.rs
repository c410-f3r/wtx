use crate::{crypto::Agreement, rng::CryptoRng};

type P256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P256Ring,
  feature = "crypto-graviola" => crate::crypto::P256Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::P256AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::P256Openssl,
  _ => crate::crypto::AgreementDummy::<(), [u8; 65], [u8; 32]>
};
type P384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P384Ring,
  feature = "crypto-graviola" => crate::crypto::P384Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::P384AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::P384Openssl,
  _ => crate::crypto::AgreementDummy::<(), [u8; 97], [u8; 48]>
};
type X25519Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::X25519Ring,
  feature = "crypto-graviola" => crate::crypto::X25519Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::X25519AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::X25519Openssl,
  _ => crate::crypto::AgreementDummy::<(), [u8; 32], [u8; 32]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct P256AgreementGlobal;

impl Agreement for P256AgreementGlobal {
  type EphemeralSecretKey = <P256Ty as Agreement>::EphemeralSecretKey;
  type PublicKey = <P256Ty as Agreement>::PublicKey;
  type SharedSecret = <P256Ty as Agreement>::SharedSecret;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    P256Ty::diffie_hellman(esk, other_participant_pk)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    P256Ty::ephemeral_secret_key(rng)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    P256Ty::public_key(esk)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct P384AgreementGlobal;

impl Agreement for P384AgreementGlobal {
  type EphemeralSecretKey = <P384Ty as Agreement>::EphemeralSecretKey;
  type PublicKey = <P384Ty as Agreement>::PublicKey;
  type SharedSecret = <P384Ty as Agreement>::SharedSecret;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    P384Ty::diffie_hellman(esk, other_participant_pk)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    P384Ty::ephemeral_secret_key(rng)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    P384Ty::public_key(esk)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct X25519Global;

impl Agreement for X25519Global {
  type EphemeralSecretKey = <X25519Ty as Agreement>::EphemeralSecretKey;
  type PublicKey = <X25519Ty as Agreement>::PublicKey;
  type SharedSecret = <X25519Ty as Agreement>::SharedSecret;

  #[inline]
  fn diffie_hellman(
    esk: Self::EphemeralSecretKey,
    other_participant_pk: &[u8],
  ) -> crate::Result<Self::SharedSecret> {
    X25519Ty::diffie_hellman(esk, other_participant_pk)
  }

  #[inline]
  fn ephemeral_secret_key<RNG>(rng: &mut RNG) -> crate::Result<Self::EphemeralSecretKey>
  where
    RNG: CryptoRng,
  {
    X25519Ty::ephemeral_secret_key(rng)
  }

  #[inline]
  fn public_key(esk: &Self::EphemeralSecretKey) -> crate::Result<Self::PublicKey> {
    X25519Ty::public_key(esk)
  }
}
