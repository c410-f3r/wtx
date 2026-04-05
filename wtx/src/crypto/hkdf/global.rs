use crate::crypto::Hkdf;

type HkdfSha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::HkdfSha256Ring,
  feature = "crypto-graviola" => crate::crypto::HkdfSha256Graviola,
  feature = "crypto-rust-crypto" => crate::crypto::HkdfSha256RustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::HkdfSha256AwsLcRs,
  _ => crate::crypto::HkdfStub::<[u8; 32]>
};
type HkdfSha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::HkdfSha384Ring,
  feature = "crypto-graviola" => crate::crypto::HkdfSha384Graviola,
  feature = "crypto-rust-crypto" => crate::crypto::HkdfSha384RustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::HkdfSha384AwsLcRs,
  _ => crate::crypto::HkdfStub::<[u8; 48]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalHkdfSha256(HkdfSha256Ty);

impl Hkdf for GlobalHkdfSha256 {
  type Digest = <HkdfSha256Ty as Hkdf>::Digest;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (digest, this) = HkdfSha256Ty::extract(salt, ikm);
    (digest, Self(this))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self(HkdfSha256Ty::from_prk(prk)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    HkdfSha256Ty::compute(data, key)
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    self.0.expand(info, okm)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalHkdfSha384(HkdfSha384Ty);

impl Hkdf for GlobalHkdfSha384 {
  type Digest = <HkdfSha384Ty as Hkdf>::Digest;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (digest, this) = HkdfSha384Ty::extract(salt, ikm);
    (digest, Self(this))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self(HkdfSha384Ty::from_prk(prk)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    HkdfSha384Ty::compute(data, key)
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    self.0.expand(info, okm)
  }
}
