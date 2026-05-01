/// Decrypts/Encrypts with AES-128 in Galois/Counter Mode using the SHA-256 hasher.
#[derive(Debug)]
pub struct Aes128GcmSha256<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Aes128GcmSha256<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "tls-aws-lc-rs")]
impl crate::tls::cipher_suite::CipherSuite for Aes128GcmSha256<crate::tls::AwsLcRs> {
  type Aead = crate::crypto::Aes128GcmAwsLcRs;
  type Hash = crate::crypto::Sha256DigestAwsLcRs;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    const { &crate::crypto::Aes128GcmAwsLcRs::new() }
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    const { &crate::crypto::Sha256DigestAwsLcRs::new() }
  }

  #[inline]
  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes128GcmSha256
  }
}

#[cfg(feature = "tls-rust-crypto")]
impl crate::tls::cipher_suite::CipherSuite for Aes128GcmSha256<crate::tls::RustCrypto> {
  type Aead = crate::crypto::Aes128GcmRustCrypto;
  type Hash = crate::crypto::Sha256DigestRustCrypto;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    const { &crate::crypto::Aes128GcmRustCrypto::new() }
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    const { &crate::crypto::Sha256DigestRustCrypto::new() }
  }

  #[inline]
  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes128GcmSha256
  }
}
