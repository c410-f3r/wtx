use crate::crypto::{CryptoError, Hmac, HmacOpenssl, HmacSha256Openssl, HmacSha384Openssl};
use openssl::{hash::MessageDigest, memcmp};

impl Hmac for HmacSha256Openssl {
  type Digest = [u8; 32];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(HmacOpenssl::new(MessageDigest::sha256(), key)?))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.signer.update(data).unwrap()
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    let mut ret = [0; 32];
    let _rslt = self.0.signer.sign(&mut ret);
    ret
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    let computed = self.digest();
    if memcmp::eq(&computed, tag) { Ok(()) } else { Err(CryptoError::HmacVerificationError.into()) }
  }
}

impl Hmac for HmacSha384Openssl {
  type Digest = [u8; 48];

  #[inline]
  fn from_key(key: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(HmacOpenssl::new(MessageDigest::sha384(), key)?))
  }

  #[inline]
  fn update(&mut self, data: &[u8]) {
    self.0.signer.update(data).unwrap()
  }

  #[inline]
  fn digest(self) -> Self::Digest {
    let mut ret = [0; 48];
    let _rslt = self.0.signer.sign(&mut ret);
    ret
  }

  #[inline]
  fn verify(self, tag: &[u8]) -> crate::Result<()> {
    let computed = self.digest();
    if memcmp::eq(&computed, tag) { Ok(()) } else { Err(CryptoError::HmacVerificationError.into()) }
  }
}
