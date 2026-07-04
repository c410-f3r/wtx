use crate::crypto::{GraviolaPrk, Hkdf, HkdfSha256Graviola, HkdfSha384Graviola};
use graviola::hashing::{HashOutput, Sha256, Sha384};

impl Hkdf for HkdfSha256Graviola {
  type Digest = HashOutput;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = GraviolaPrk::<Sha256>::extract(salt, ikm);
    (lhs, HkdfSha256Graviola(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self(GraviolaPrk::<Sha256>::new(prk)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    Ok(GraviolaPrk::<Sha256>::compute(data, key))
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    self.0.expand(info, okm)
  }
}

impl Hkdf for HkdfSha384Graviola {
  type Digest = HashOutput;

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let (lhs, rhs) = GraviolaPrk::<Sha384>::extract(salt, ikm);
    (lhs, HkdfSha384Graviola(rhs))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self(GraviolaPrk::<Sha384>::new(prk)?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    Ok(GraviolaPrk::<Sha384>::compute(data, key))
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    self.0.expand(info, okm)
  }
}
