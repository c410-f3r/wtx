#[cfg(any(feature = "aws-lc-rs", feature = "rust-crypto"))]
use crate::tls::cipher_suite::{CipherSuite, CipherSuiteTy};
use core::marker::PhantomData;

/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Aes256GcmSha384<CS>(PhantomData<CS>);

#[cfg(feature = "aws-lc-rs")]
impl CipherSuite for Aes256GcmSha384<crate::tls::AwsLc> {
  const TY: CipherSuiteTy = CipherSuiteTy::Aes256GcmSha384;

  type Aead = crate::tls::cipher_suite::Aes256GcmAwsLcRs;
  type Hash = crate::tls::cipher_suite::Sha384AwsLcRs;
}

#[cfg(feature = "rust-crypto")]
impl CipherSuite for Aes256GcmSha384<crate::tls::RustCrypto> {
  const TY: CipherSuiteTy = CipherSuiteTy::Aes256GcmSha384;

  type Aead = aes_gcm::Aes256Gcm;
  type Hash = sha2::Sha384;
}
