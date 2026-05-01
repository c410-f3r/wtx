use crate::{crypto::Signature, rng::CryptoRng};

type Ed25519Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Ed25519Ring,
  feature = "crypto-graviola" => crate::crypto::Ed25519Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Ed25519AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Ed25519Openssl,
  _ => crate::crypto::SignatureDummy::<crate::crypto::SignKeyDummy, [u8; 64]>
};
type P256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P256Ring,
  feature = "crypto-graviola" => crate::crypto::P256Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::P256AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::P256Openssl,
  _ => crate::crypto::SignatureDummy::<crate::crypto::SignKeyDummy, [u8; 64]>
};
type P384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P384Ring,
  feature = "crypto-graviola" => crate::crypto::P384Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::P384AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::P384Openssl,
  _ => crate::crypto::SignatureDummy::<crate::crypto::SignKeyDummy, [u8; 96]>
};
type RsaPssRsaeSha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::RsaPssRsaeSha256Ring,
  feature = "crypto-graviola" => crate::crypto::RsaPssRsaeSha256Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::RsaPssRsaeSha256AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::RsaPssRsaeSha256Openssl,
  _ => crate::crypto::SignatureDummy::<crate::crypto::SignKeyDummy, [u8; 0]>
};
type RsaPssRsaeSha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::RsaPssRsaeSha384Ring,
  feature = "crypto-graviola" => crate::crypto::RsaPssRsaeSha384Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::RsaPssRsaeSha384AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::RsaPssRsaeSha384Openssl,
  _ => crate::crypto::SignatureDummy::<crate::crypto::SignKeyDummy, [u8; 0]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Ed25519Global;

impl Signature for Ed25519Global {
  type SignKey = <Ed25519Ty as Signature>::SignKey;
  type SignOutput = <Ed25519Ty as Signature>::SignOutput;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    Ed25519Ty::sign(rng, sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ed25519Ty::validate(pk, msg, signature)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct P256SignatureGlobal;

impl Signature for P256SignatureGlobal {
  type SignKey = <P256Ty as Signature>::SignKey;
  type SignOutput = <P256Ty as Signature>::SignOutput;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    P256Ty::sign(rng, sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    P256Ty::validate(pk, msg, signature)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct P384SignatureGlobal;

impl Signature for P384SignatureGlobal {
  type SignKey = <P384Ty as Signature>::SignKey;
  type SignOutput = <P384Ty as Signature>::SignOutput;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    P384Ty::sign(rng, sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    P384Ty::validate(pk, msg, signature)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct RsaPssRsaeSha256Global;

impl Signature for RsaPssRsaeSha256Global {
  type SignKey = <RsaPssRsaeSha256Ty as Signature>::SignKey;
  type SignOutput = <RsaPssRsaeSha256Ty as Signature>::SignOutput;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    RsaPssRsaeSha256Ty::sign(rng, sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    RsaPssRsaeSha256Ty::validate(pk, msg, signature)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct RsaPssRsaeSha384Global;

impl Signature for RsaPssRsaeSha384Global {
  type SignKey = <RsaPssRsaeSha384Ty as Signature>::SignKey;
  type SignOutput = <RsaPssRsaeSha384Ty as Signature>::SignOutput;

  #[inline]
  fn sign<RNG>(
    rng: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    RsaPssRsaeSha384Ty::sign(rng, sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    RsaPssRsaeSha384Ty::validate(pk, msg, signature)
  }
}
