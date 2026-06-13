use crate::crypto::Hash;

type Sha1Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha1HashRing,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha1HashAwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Sha1HashOpenssl,
  _ => crate::crypto::HashDummy::<[u8; 20]>
};
type Sha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha256HashRing,
  feature = "crypto-graviola" => crate::crypto::Sha256HashGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha256HashAwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Sha256HashOpenssl,
  _ => crate::crypto::HashDummy::<[u8; 32]>
};
type Sha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha384HashRing,
  feature = "crypto-graviola" => crate::crypto::Sha384HashGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha384HashAwsLcRs,
  feature = "crypto-openssl" => crate::crypto::Sha384HashOpenssl,
  _ => crate::crypto::HashDummy::<[u8; 48]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Sha1HashGlobal;

impl Hash for Sha1HashGlobal {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha1Ty::digest(data)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Sha256HashGlobal;

impl Hash for Sha256HashGlobal {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha256Ty::digest(data)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Sha384HashGlobal;

impl Hash for Sha384HashGlobal {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha384Ty::digest(data)
  }
}
