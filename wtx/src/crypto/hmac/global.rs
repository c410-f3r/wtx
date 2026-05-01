use crate::crypto::Hmac;

type HmacSha256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::HmacSha256Ring,
  feature = "crypto-graviola" => crate::crypto::HmacSha256Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::HmacSha256AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::HmacSha256Openssl,
  _ => crate::crypto::HmacDummy::<[u8; 32]>
};

type HmacSha384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::HmacSha384Ring,
  feature = "crypto-graviola" => crate::crypto::HmacSha384Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::HmacSha384AwsLcRs,
  feature = "crypto-openssl" => crate::crypto::HmacSha384Openssl,
  _ => crate::crypto::HmacDummy::<[u8; 48]>
};

/// A structure that delegates HMAC-SHA-256 execution to the selected crypto backend.
#[derive(Debug)]
pub struct HmacSha256Global(HmacSha256Ty);

impl Hmac for HmacSha256Global {
  type Digest = <HmacSha256Ty as Hmac>::Digest;

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self(HmacSha256Ty::from_key(key)?))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    self.0.digest()
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    self.0.verify(tag)
  }
}

/// A structure that delegates HMAC-SHA-384 execution to the selected crypto backend.
#[derive(Debug)]
pub struct HmacSha384Global(HmacSha384Ty);

impl Hmac for HmacSha384Global {
  type Digest = <HmacSha384Ty as Hmac>::Digest;

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self(HmacSha384Ty::from_key(key)?))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data);
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    self.0.digest()
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    self.0.verify(tag)
  }
}
