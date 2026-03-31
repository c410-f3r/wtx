use crate::misc::unlikely_elem;
use graviola::hashing::Hash as _;

impl crate::crypto::Hash for crate::crypto::Sha256DigestGraviola {
  type Digest = [u8; 32];

  #[inline]
  fn digest(data: &[u8]) -> Self::Digest {
    let rlst = graviola::hashing::Sha256::hash(data);
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }
}

impl crate::crypto::Hash for crate::crypto::Sha384DigestGraviola {
  type Digest = [u8; 48];

  #[inline]
  fn digest(data: &[u8]) -> Self::Digest {
    let rlst = graviola::hashing::Sha384::hash(data);
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }
}
