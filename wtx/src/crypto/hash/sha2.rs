use crate::crypto::{Hash, Sha256DigestRustCrypto, Sha384DigestRustCrypto};
use sha2::Digest;

impl Hash for Sha256DigestRustCrypto {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl Iterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = <sha2::Sha256 as Digest>::new();
    for elem in data {
      ctx.update(elem);
    }
    ctx.finalize().into()
  }
}

impl Hash for Sha384DigestRustCrypto {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl Iterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = <sha2::Sha384 as Digest>::new();
    for elem in data {
      ctx.update(elem);
    }
    ctx.finalize().into()
  }
}
