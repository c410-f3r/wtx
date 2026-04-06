use crate::crypto::Hash;

type Sha1Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha1DigestRing,
  feature = "crypto-rust-crypto" => crate::crypto::Sha1DigestRustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha1DigestAwsLcRs,
  _ => crate::crypto::HashStub::<[u8; 20]>
};
type Sha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha256DigestRing,
  feature = "crypto-graviola" => crate::crypto::Sha256DigestGraviola,
  feature = "crypto-rust-crypto" => crate::crypto::Sha256DigestRustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha256DigestAwsLcRs,
  _ => crate::crypto::HashStub::<[u8; 32]>
};
type Sha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha384DigestRing,
  feature = "crypto-graviola" => crate::crypto::Sha384DigestGraviola,
  feature = "crypto-rust-crypto" => crate::crypto::Sha384DigestRustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha384DigestAwsLcRs,
  _ => crate::crypto::HashStub::<[u8; 48]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalSha1;

impl Hash for GlobalSha1 {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha1Ty::digest(data)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalSha256;

impl Hash for GlobalSha256 {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha256Ty::digest(data)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalSha386;

impl Hash for GlobalSha386 {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha384Ty::digest(data)
  }
}
