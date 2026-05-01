use crate::crypto::Hash;

type Sha1Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha1DigestRing,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha1DigestAwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Sha1DigestOpenssl,
  _ => crate::crypto::HashDummy::<[u8; 20]>
};
type Sha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha256DigestRing,
  feature = "crypto-graviola" => crate::crypto::Sha256DigestGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha256DigestAwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Sha256DigestOpenssl,
  _ => crate::crypto::HashDummy::<[u8; 32]>
};
type Sha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha384DigestRing,
  feature = "crypto-graviola" => crate::crypto::Sha384DigestGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha384DigestAwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Sha384DigestOpenssl,
  _ => crate::crypto::HashDummy::<[u8; 48]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Sha1DigestGlobal;

impl Hash for Sha1DigestGlobal {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha1Ty::digest(data)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Sha256DigestGlobal;

impl Hash for Sha256DigestGlobal {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha256Ty::digest(data)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Sha386DigestGlobal;

impl Hash for Sha386DigestGlobal {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha384Ty::digest(data)
  }
}
