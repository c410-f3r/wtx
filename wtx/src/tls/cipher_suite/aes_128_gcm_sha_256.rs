#[cfg(any(feature = "aws-lc-rs", feature = "rust-crypto"))]
use crate::tls::cipher_suite::{CipherSuite, CipherSuiteTy};

/// Decrypts/Encrypts with AES-128 in Galois/Counter Mode using the SHA-256 hasher.
#[derive(Debug)]
pub struct Aes128GcmSha256<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Aes128GcmSha256<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "aws-lc-rs")]
impl CipherSuite for Aes128GcmSha256<crate::tls::AwsLcRs> {
  type Aead = crate::tls::cipher_suite::Aes128GcmAwsLcRs;
  type Hash = crate::tls::cipher_suite::Sha256AwsLcRs;
  type Hkdf = aws_lc_rs::hkdf::Algorithm;

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Aes128GcmSha256
  }
}

#[cfg(feature = "rust-crypto")]
impl CipherSuite for Aes128GcmSha256<crate::tls::RustCrypto> {
  type Aead = aes_gcm::Aes128Gcm;
  type Hash = sha2::Sha256;
  type Hkdf = hkdf::Hkdf<Self::Hash>;

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Aes128GcmSha256
  }
}
