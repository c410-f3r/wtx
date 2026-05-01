// The algorithm is infallible but somehow openssl's internals can throw errors

#![allow(
  clippy::unwrap_used,
  reason = "it is not worth changing the signature because of one backend"
)]

use crate::{
  crypto::{Hash, Sha1DigestOpenssl, Sha256DigestOpenssl, Sha384DigestOpenssl},
  misc::unlikely_elem,
};
use openssl::hash::{Hasher, MessageDigest};

impl Hash for Sha1DigestOpenssl {
  type Digest = [u8; 20];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    local_digest(data, [0; 20], MessageDigest::sha1()).unwrap()
  }
}

impl Hash for Sha256DigestOpenssl {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    local_digest(data, [0; 32], MessageDigest::sha256()).unwrap()
  }
}

impl Hash for Sha384DigestOpenssl {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl IntoIterator<Item = &'data [u8]>) -> Self::Digest {
    local_digest(data, [0; 48], MessageDigest::sha384()).unwrap()
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
  let rslt = hasher.finish()?;
  Ok(if let Ok(elem) = rslt.as_ref().try_into() { elem } else { unlikely_elem(default) })
}
