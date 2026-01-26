/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Aes256GcmSha384<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Aes256GcmSha384<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "aws-lc-rs")]
impl crate::tls::cipher_suite::CipherSuite for Aes256GcmSha384<crate::tls::AwsLcRs> {
  type Aead = crate::tls::protocol::cipher_suite_wrappers::Aes256GcmAwsLcRs;
  type Hash = crate::tls::protocol::cipher_suite_wrappers::Sha384AwsLcRs;
  type Hkdf = aws_lc_rs::hkdf::Algorithm;

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes256GcmSha384
  }
}

#[cfg(feature = "rust-crypto")]
impl crate::tls::cipher_suite::CipherSuite for Aes256GcmSha384<crate::tls::RustCrypto> {
  type Aead = aes_gcm::Aes256Gcm;
  type Hash = sha2::Sha384;
  type Hkdf = hkdf::Hkdf<Self::Hash>;

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes256GcmSha384
  }
}
