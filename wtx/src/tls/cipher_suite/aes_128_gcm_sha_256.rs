#[cfg(any(feature = "aws-lc-rs", feature = "rust-crypto"))]
use crate::tls::cipher_suite::{CipherSuite, CipherSuiteTy};

/// Decrypts/Encrypts with AES-128 in Galois/Counter Mode using the SHA-256 hasher.
#[derive(Debug)]
pub struct Aes128GcmSha256<CS>(core::marker::PhantomData<CS>);

#[cfg(feature = "aws-lc-rs")]
impl CipherSuite for Aes128GcmSha256<crate::tls::AwsLc> {
  const TY: CipherSuiteTy = CipherSuiteTy::Aes128GcmSha256;

  type Aead = crate::tls::cipher_suite::Aes128GcmAwsLcRs;
  type Hash = crate::tls::cipher_suite::Sha256AwsLcRs;
}

#[cfg(feature = "rust-crypto")]
impl CipherSuite for Aes128GcmSha256<crate::tls::RustCrypto> {
  const TY: CipherSuiteTy = CipherSuiteTy::Aes128GcmSha256;

  type Aead = aes_gcm::Aes128Gcm;
  type Hash = sha2::Sha256;
}
