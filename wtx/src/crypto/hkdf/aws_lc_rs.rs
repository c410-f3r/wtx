use crate::{
  crypto::{CryptoError, Hkdf, HkdfSha256AwsLcRs, HkdfSha384AwsLcRs},
  misc::unlikely_elem,
};
use aws_lc_rs::{
  hkdf::{self, HKDF_SHA256, HKDF_SHA384, Prk},
  hmac::{self, Context, HMAC_SHA256, HMAC_SHA384, Key},
};

impl Hkdf for HkdfSha256AwsLcRs {
  type Digest = [u8; 32];

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = local_extract::<32>(HKDF_SHA256, HMAC_SHA256, salt, ikm);
    (lhs, Self::new(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(Prk::new_less_safe(HKDF_SHA256, prk)))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    local_compute(HMAC_SHA256, data, key)
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    local_expand(HKDF_SHA256, info, okm, &self.0)
  }
}

impl Hkdf for HkdfSha384AwsLcRs {
  type Digest = [u8; 48];

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = local_extract::<48>(HKDF_SHA384, HMAC_SHA384, salt, ikm);
    (lhs, Self::new(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(Prk::new_less_safe(HKDF_SHA384, prk)))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    local_compute(HMAC_SHA384, data, key)
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    local_expand(HKDF_SHA384, info, okm, &self.0)
  }
}

common_hkdf_functions!();
