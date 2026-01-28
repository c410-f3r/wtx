use crate::{collection::ArrayVectorU8, tls::MAX_HASH_LEN};

/// HMAC-based Key Derivation Function
pub(crate) trait Hkdf: Sized {
  fn from_prk(prk: &[u8]) -> crate::Result<Self>;

  fn compute_one(data: &[u8], key: &[u8]) -> ArrayVectorU8<u8, MAX_HASH_LEN>;

  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()>;

  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self);
}

impl Hkdf for () {
  fn from_prk(_: &[u8]) -> crate::Result<Self> {
    Ok(())
  }

  fn compute_one(data: &[u8], key: &[u8]) -> ArrayVectorU8<u8, MAX_HASH_LEN> {
    ArrayVectorU8::new()
  }

  fn expand(&self, _: &[u8], _: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }

  fn extract(_: Option<&[u8]>, _: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    (ArrayVectorU8::new(), ())
  }
}

#[cfg(feature = "aws-lc-rs")]
impl Hkdf for aws_lc_rs::hkdf::Algorithm {
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    todo!()
  }

  fn compute_one(data: &[u8], key: &[u8]) -> ArrayVectorU8<u8, MAX_HASH_LEN> {
    ArrayVectorU8::new()
  }

  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }

  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    todo!()
  }
}

#[cfg(feature = "rust-crypto")]
impl<H> Hkdf for hkdf::Hkdf<H>
where
  H: hmac::EagerHash,
{
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    todo!()
  }

  fn compute_one(data: &[u8], key: &[u8]) -> ArrayVectorU8<u8, MAX_HASH_LEN> {
    ArrayVectorU8::new()
  }

  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }

  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    todo!()
  }
}
