use crate::crypto::Hash;

type Sha1Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha1Ring,
  feature = "crypto-alr" => crate::crypto::Sha1Alr,
  feature = "crypto-ruco" => crate::crypto::Sha1Ruco,
  _ => crate::crypto::HashDummy::<[u8; 20]>,
};
type Sha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha256Ring,
  feature = "crypto-graviola" => crate::crypto::Sha256Graviola,
  feature = "crypto-alr" => crate::crypto::Sha256Alr,
  feature = "crypto-ruco" => crate::crypto::Sha256Ruco,
  _ => crate::crypto::HashDummy::<[u8; 32]>,
};
type Sha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha384Ring,
  feature = "crypto-graviola" => crate::crypto::Sha384Graviola,
  feature = "crypto-alr" => crate::crypto::Sha384Alr,
  feature = "crypto-ruco" => crate::crypto::Sha384Ruco,
  _ => crate::crypto::HashDummy::<[u8; 48]>,
};
type Sha512Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Sha512Ring,
  feature = "crypto-graviola" => crate::crypto::Sha512Graviola,
  feature = "crypto-alr" => crate::crypto::Sha512Alr,
  feature = "crypto-ruco" => crate::crypto::Sha512Ruco,
  _ => crate::crypto::HashDummy::<[u8; 64]>,
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Debug)]
pub struct Sha1Global(Sha1Ty);

impl Hash for Sha1Global {
  type Digest = [u8; 20];

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
pub struct Sha256Global(Sha256Ty);

impl Hash for Sha256Global {
  type Digest = [u8; 32];

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
pub struct Sha384Global(Sha384Ty);

impl Hash for Sha384Global {
  type Digest = [u8; 48];

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

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Debug)]
pub struct Sha512Global(Sha512Ty);

impl Hash for Sha512Global {
  type Digest = [u8; 64];

  #[inline]
  fn new() -> Self {
    Self(<Sha512Ty as Hash>::new())
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
