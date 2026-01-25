#[cfg(any(feature = "aws-lc-rs", feature = "rust-crypto"))]
use crate::tls::cipher_suite::{CipherSuite, CipherSuiteTy};
use core::marker::PhantomData;

/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Aes256GcmSha384<CS>(PhantomData<CS>);

impl<CS> Default for Aes256GcmSha384<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "aws-lc-rs")]
impl CipherSuite for Aes256GcmSha384<crate::tls::AwsLcRs> {
  type Aead = crate::tls::cipher_suite::Aes256GcmAwsLcRs;
  type Hash = crate::tls::cipher_suite::Sha384AwsLcRs;
  type Hkdf = aws_lc_rs::hkdf::Algorithm;

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Aes256GcmSha384
  }
}

#[cfg(feature = "rust-crypto")]
impl CipherSuite for Aes256GcmSha384<crate::tls::RustCrypto> {
  type Aead = aes_gcm::Aes256Gcm;
  type Hash = sha2::Sha384;
  type Hkdf = hkdf::Hkdf<Self::Hash>;

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Aes256GcmSha384
  }
}
