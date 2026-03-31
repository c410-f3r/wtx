use crate::crypto::{Hash, Sha256DigestRustCrypto, Sha384DigestRustCrypto};

impl Hash for Sha256DigestRustCrypto {
  type Digest = [u8; 32];

  #[inline]
  fn digest(data: &[u8]) -> Self::Digest {
    <sha2::Sha256 as sha2::Digest>::digest(data).into()
  }
}

impl Hash for Sha384DigestRustCrypto {
  type Digest = [u8; 48];

  #[inline]
  fn digest(data: &[u8]) -> Self::Digest {
    <sha2::Sha384 as sha2::Digest>::digest(data).into()
  }
}
