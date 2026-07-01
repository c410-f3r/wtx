use crate::crypto::{Hkdf as _, HkdfSha256Global, HkdfSha384Global};

#[derive(Debug)]
pub(crate) enum TlsHkdf {
  Sha256(HkdfSha256Global),
  Sha384(HkdfSha384Global),
}

impl TlsHkdf {
  #[inline]
  pub(crate) fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    match self {
      Self::Sha256(el) => el.expand(info, okm),
      Self::Sha384(el) => el.expand(info, okm),
    }
  }
}
