use crate::{
  crypto::{Hmac as _, HmacSha256Global, HmacSha384Global},
  tls::tls_hash::TlsDigest,
};

#[derive(Debug)]
pub(crate) enum TlsHmac {
  Sha256(HmacSha256Global),
  Sha384(HmacSha384Global),
}

impl TlsHmac {
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

  #[inline]
  pub(crate) fn verify(self, tag: &[u8]) -> crate::Result<()> {
    match self {
      Self::Sha256(el) => el.verify(tag),
      Self::Sha384(el) => el.verify(tag),
    }
  }
}
