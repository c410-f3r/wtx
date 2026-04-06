use crate::crypto::{Hash, Sha1DigestRustCrypto};
use sha1::Digest;

impl Hash for Sha1DigestRustCrypto {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    let mut ctx = <sha1::Sha1 as Digest>::new();
    for elem in data {
      ctx.update(elem);
    }
    ctx.finalize().into()
  }
}
