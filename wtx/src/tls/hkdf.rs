use crate::{collection::ArrayVectorU8, tls::MAX_HASH_LEN};

/// HMAC-based Key Derivation Function
pub(crate) trait Hkdf: Sized {
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self);

  fn from_prk(prk: &[u8]) -> crate::Result<Self>;

  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()>;
}

impl Hkdf for () {
  fn extract(_: Option<&[u8]>, _: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    (ArrayVectorU8::new(), ())
  }

  fn from_prk(_: &[u8]) -> crate::Result<Self> {
    Ok(())
  }

  fn expand(&self, _: &[u8], _: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "aws-lc-rs")]
impl Hkdf for aws_lc_rs::hkdf::Algorithm {
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    todo!()
  }

  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    todo!()
  }

  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "rust-crypto")]
impl<H> Hkdf for hkdf::Hkdf<H>
where
  H: hmac::EagerHash,
{
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    todo!()
  }

  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    todo!()
  }

  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }
}
