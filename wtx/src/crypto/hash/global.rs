use crate::crypto::Hash;

type Sha1Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha1HashRing,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha1HashAwsLcRs,
  _ => crate::crypto::HashDummy::<[u8; 20]>
};
type Sha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha256HashRing,
  feature = "crypto-graviola" => crate::crypto::Sha256HashGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha256HashAwsLcRs,
  _ => crate::crypto::HashDummy::<[u8; 32]>
};
type Sha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha384HashRing,
  feature = "crypto-graviola" => crate::crypto::Sha384HashGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Sha384HashAwsLcRs,
  _ => crate::crypto::HashDummy::<[u8; 48]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Debug)]
pub struct Sha1HashGlobal(Sha1Ty);

impl Hash for Sha1HashGlobal {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha1Ty::digest(data)
  }

  #[inline]
  fn new() -> Self {
    Self(<Sha1Ty as Hash>::new())
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    self.0.finalize()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Debug)]
pub struct Sha256HashGlobal(Sha256Ty);

impl Hash for Sha256HashGlobal {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha256Ty::digest(data)
  }

  #[inline]
  fn new() -> Self {
    Self(<Sha256Ty as Hash>::new())
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    self.0.finalize()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Debug)]
pub struct Sha384HashGlobal(Sha384Ty);

impl Hash for Sha384HashGlobal {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    Sha384Ty::digest(data)
  }

  #[inline]
  fn new() -> Self {
    Self(<Sha384Ty as Hash>::new())
  }

  #[inline]
  fn finalize(self) -> Self::Digest {
    self.0.finalize()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }
}
