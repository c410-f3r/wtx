/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Aes256GcmSha384<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Aes256GcmSha384<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "tls-aws-lc-rs")]
impl crate::tls::cipher_suite::CipherSuite for Aes256GcmSha384<crate::tls::AwsLcRs> {
  type Aead = crate::crypto::Aes256GcmAwsLcRs;
  type Hash = crate::crypto::Sha384DigestAwsLcRs;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    const { &crate::crypto::Aes256GcmAwsLcRs::new() }
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    const { &crate::crypto::Sha384DigestAwsLcRs::new() }
  }

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes256GcmSha384
  }
}

#[cfg(feature = "tls-rust-crypto")]
impl crate::tls::cipher_suite::CipherSuite for Aes256GcmSha384<crate::tls::RustCrypto> {
  type Aead = crate::crypto::Aes256GcmRustCrypto;
  type Hash = crate::crypto::Sha384DigestRustCrypto;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    const { &crate::crypto::Aes256GcmRustCrypto::new() }
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    const { &crate::crypto::Sha384DigestRustCrypto::new() }
  }

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes256GcmSha384
  }
}
