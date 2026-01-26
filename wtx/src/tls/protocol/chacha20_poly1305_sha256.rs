/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Chacha20Poly1305Sha256<CS>(core::marker::PhantomData<CS>);

impl<CS> Default for Chacha20Poly1305Sha256<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "aws-lc-rs")]
impl crate::tls::cipher_suite::CipherSuite for Chacha20Poly1305Sha256<crate::tls::AwsLcRs> {
  type Aead = crate::tls::protocol::cipher_suite_wrappers::Chacha20Poly1305AwsLcRs;
  type Hash = crate::tls::protocol::cipher_suite_wrappers::Sha256AwsLcRs;
  type Hkdf = aws_lc_rs::hkdf::Algorithm;

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Chacha20Poly1305Sha256
  }
}

#[cfg(feature = "rust-crypto")]
impl crate::tls::cipher_suite::CipherSuite for Chacha20Poly1305Sha256<crate::tls::RustCrypto> {
  type Aead = chacha20poly1305::ChaCha20Poly1305;
  type Hash = sha2::Sha256;
  type Hkdf = hkdf::Hkdf<Self::Hash>;

  fn ty(&self) -> crate::tls::CipherSuiteTy {
    crate::tls::CipherSuiteTy::Chacha20Poly1305Sha256
  }
}
