use crate::{crypto::Agreement, rng::CryptoRng};

type P256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P256Ring,
  feature = "crypto-graviola" => crate::crypto::P256Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::P256AwsLcRs,
  feature = "crypto-ruco" => crate::crypto::P256Ruco,
  _ => crate::crypto::AgreementDummy::<[u8; 65], [u8; 32]>
};
type P384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P384Ring,
  feature = "crypto-graviola" => crate::crypto::P384Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::P384AwsLcRs,
  feature = "crypto-ruco" => crate::crypto::P384Ruco,
  _ => crate::crypto::AgreementDummy::<[u8; 97], [u8; 48]>
};
type X25519Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::X25519Ring,
  feature = "crypto-graviola" => crate::crypto::X25519Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::X25519AwsLcRs,
  feature = "crypto-ruco" => crate::crypto::X25519Ruco,
  _ => crate::crypto::AgreementDummy::<[u8; 32], [u8; 32]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct P256AgreementGlobal(P256Ty);

impl Agreement for P256AgreementGlobal {
  type PublicKey = <P256Ty as Agreement>::PublicKey;
  type SharedSecret = <P256Ty as Agreement>::SharedSecret;

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(P256Ty::generate(rng)?))
  }

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    self.0.diffie_hellman(other_participant_pk)
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    self.0.public_key()
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct P384AgreementGlobal(P384Ty);

impl Agreement for P384AgreementGlobal {
  type PublicKey = <P384Ty as Agreement>::PublicKey;
  type SharedSecret = <P384Ty as Agreement>::SharedSecret;

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(P384Ty::generate(rng)?))
  }

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    self.0.diffie_hellman(other_participant_pk)
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    self.0.public_key()
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct X25519Global(X25519Ty);

impl Agreement for X25519Global {
  type PublicKey = <X25519Ty as Agreement>::PublicKey;
  type SharedSecret = <X25519Ty as Agreement>::SharedSecret;

  #[inline]
  fn generate<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self(X25519Ty::generate(rng)?))
  }

  #[inline]
  fn diffie_hellman(self, other_participant_pk: &[u8]) -> crate::Result<Self::SharedSecret> {
    self.0.diffie_hellman(other_participant_pk)
  }

  #[inline]
  fn public_key(&self) -> crate::Result<Self::PublicKey> {
    self.0.public_key()
  }
}
