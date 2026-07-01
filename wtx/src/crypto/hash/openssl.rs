// The algorithm is infallible but somehow openssl's internals can throw errors

#![expect(
  clippy::unwrap_used,
  reason = "it is not worth changing the signature because of one backend"
)]

use crate::{
  crypto::{Hash, Sha1HashOpenssl, Sha256HashOpenssl, Sha384HashOpenssl},
  misc::unlikely_elem,
};
use openssl::hash::{Hasher, MessageDigest};

impl Hash for Sha1HashOpenssl {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    local_digest(data, [0; 20], MessageDigest::sha1()).unwrap()
  }

  #[inline]
  fn new() -> Self {
    Self(Hasher::new(MessageDigest::sha1()).unwrap())
  }

  #[inline]
  fn finalize(mut self) -> Self::Digest {
    local_finalize(&mut self.0, [0; 20]).unwrap()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data).unwrap();
  }
}

impl Hash for Sha256HashOpenssl {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    local_digest(data, [0; 32], MessageDigest::sha256()).unwrap()
  }

  #[inline]
  fn new() -> Self {
    Self(Hasher::new(MessageDigest::sha256()).unwrap())
  }

  #[inline]
  fn finalize(mut self) -> Self::Digest {
    local_finalize(&mut self.0, [0; 32]).unwrap()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data).unwrap();
  }
}

impl Hash for Sha384HashOpenssl {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    local_digest(data, [0; 48], MessageDigest::sha384()).unwrap()
  }

  #[inline]
  fn new() -> Self {
    Self(Hasher::new(MessageDigest::sha384()).unwrap())
  }

  #[inline]
  fn finalize(mut self) -> Self::Digest {
    local_finalize(&mut self.0, [0; 48]).unwrap()
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.update(data).unwrap();
  }
}

fn local_digest<'data, const N: usize>(
  data: impl IntoIterator<Item = &'data [u8]>,
  default: [u8; N],
  digest: MessageDigest,
) -> crate::Result<[u8; N]> {
  let mut hasher = Hasher::new(digest)?;
  for elem in data {
    hasher.update(elem)?;
  }
  local_finalize(&mut hasher, default)
}

fn local_finalize<const N: usize>(hasher: &mut Hasher, default: [u8; N]) -> crate::Result<[u8; N]> {
  let rslt = hasher.finish()?;
  Ok(if let Ok(elem) = rslt.as_ref().try_into() { elem } else { unlikely_elem(default) })
}
