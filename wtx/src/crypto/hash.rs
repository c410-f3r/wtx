use crate::{collection::ArrayVectorU8, crypto::MAX_HASH_LEN};

/// Maps data of arbitrary size into a fixed-size value.
pub trait Hash {
  fn digest(data: &[u8], buffer: &mut ArrayVectorU8<u8, MAX_HASH_LEN>);
}

impl Hash for () {
  fn digest(_: &[u8], _: &mut ArrayVectorU8<u8, MAX_HASH_LEN>) {}
}

#[cfg(feature = "aws-lc-rs")]
impl Hash for crate::crypto::Sha256DigestAwsLcRs {
  fn digest(data: &[u8], buffer: &mut ArrayVectorU8<u8, MAX_HASH_LEN>) {}
}
#[cfg(feature = "aws-lc-rs")]
impl Hash for crate::crypto::Sha384DigestAwsLcRs {
  fn digest(data: &[u8], buffer: &mut ArrayVectorU8<u8, MAX_HASH_LEN>) {}
}

#[cfg(feature = "rust-crypto")]
impl Hash for sha2::Sha256 {
  fn digest(data: &[u8], buffer: &mut ArrayVectorU8<u8, MAX_HASH_LEN>) {
    let hash = <sha2::Sha256 as sha2::Digest>::digest(data);
    let _rslt = buffer.extend_from_copyable_slice(hash.as_ref());
  }
}
#[cfg(feature = "rust-crypto")]
impl Hash for sha2::Sha384 {
  fn digest(data: &[u8], buffer: &mut ArrayVectorU8<u8, MAX_HASH_LEN>) {
    let hash = <sha2::Sha384 as sha2::Digest>::digest(data);
    let _rslt = buffer.extend_from_copyable_slice(hash.as_ref());
  }
}
