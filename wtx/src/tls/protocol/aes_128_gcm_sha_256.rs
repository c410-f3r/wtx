/// Decrypts/Encrypts with AES-128 in Galois/Counter Mode using the SHA-256 hasher.
#[derive(Debug)]
pub struct Aes128GcmSha256<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Aes128GcmSha256<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "aws-lc-rs")]
impl crate::tls::cipher_suite::CipherSuite for Aes128GcmSha256<crate::tls::AwsLcRs> {
  type Aead = crate::crypto::Aes256GcmAesGcm;
  type Hash = crate::crypto::Sha256DigestAwsLcRs;
  type Hkdf = crate::crypto::Sha256HkdfAwsLcRs;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    &crate::crypto::Aes256GcmAesGcm::new()
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    &crate::crypto::Sha256DigestAwsLcRs::new()
  }

  #[inline]
  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes128GcmSha256
  }
}

#[cfg(feature = "rust-crypto")]
impl crate::tls::cipher_suite::CipherSuite for Aes128GcmSha256<crate::tls::RustCrypto> {
  type Aead = aes_gcm::Aes128Gcm;
  type Hash = sha2::Sha256;
  type Hkdf = hkdf::Hkdf<Self::Hash>;

  #[inline]
  fn aead(&self) -> &Self::Aead {
    self
  }

  #[inline]
  fn hash(&self) -> &Self::Hash {
    self
  }

  #[inline]
  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Aes128GcmSha256
  }
}
