use crate::{
  crypto::{Hash as _, Sha256HashGlobal, Sha384HashGlobal},
  misc::Lease,
};

#[derive(Debug)]
pub(crate) enum TlsDigest {
  Sha256([u8; 32]),
  Sha384([u8; 48]),
}

impl Lease<[u8]> for TlsDigest {
  #[inline]
  fn lease(&self) -> &[u8] {
    match self {
      TlsDigest::Sha256(el) => el,
      TlsDigest::Sha384(el) => el,
    }
  }
}

impl Default for TlsDigest {
  #[inline]
  fn default() -> Self {
    Self::Sha256([0; 32])
  }
}

#[derive(Clone, Debug)]
pub(crate) enum TlsHash {
  Sha256(Sha256HashGlobal),
  Sha384(Sha384HashGlobal),
}

impl TlsHash {
  #[inline]
  pub(crate) fn finalize(self) -> TlsDigest {
    match self {
      Self::Sha256(el) => TlsDigest::Sha256(el.finalize()),
      Self::Sha384(el) => TlsDigest::Sha384(el.finalize()),
    }
  }

  #[inline]
  pub(crate) fn update(&mut self, data: &[u8]) {
    match self {
      Self::Sha256(el) => el.update(data),
      Self::Sha384(el) => el.update(data),
    }
  }
}
