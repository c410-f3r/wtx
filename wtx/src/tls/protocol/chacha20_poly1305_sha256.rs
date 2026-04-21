/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Chacha20Poly1305Sha256<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Chacha20Poly1305Sha256<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "tls-aws-lc-rs")]
impl crate::tls::cipher_suite::CipherSuite for Chacha20Poly1305Sha256<crate::tls::AwsLcRs> {
  type Aead = crate::crypto::Chacha20Poly1305AwsLcRs;
  type Hash = crate::crypto::Sha256DigestAwsLcRs;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    const { &crate::crypto::Chacha20Poly1305AwsLcRs::new() }
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    const { &crate::crypto::Sha256DigestAwsLcRs::new() }
  }

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Chacha20Poly1305Sha256
  }
}

#[cfg(feature = "tls-rust-crypto")]
impl crate::tls::cipher_suite::CipherSuite for Chacha20Poly1305Sha256<crate::tls::RustCrypto> {
  type Aead = crate::crypto::Chacha20Poly1305RustCrypto;
  type Hash = crate::crypto::Sha256DigestRustCrypto;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    const { &crate::crypto::Chacha20Poly1305RustCrypto::new() }
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    const { &crate::crypto::Sha256DigestRustCrypto::new() }
  }

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Chacha20Poly1305Sha256
  }
}
