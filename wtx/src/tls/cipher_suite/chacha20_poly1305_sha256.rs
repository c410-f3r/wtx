#[cfg(any(feature = "aws-lc-rs", feature = "rust-crypto"))]
use crate::tls::cipher_suite::{CipherSuite, CipherSuiteTy};
use core::marker::PhantomData;

/// Decrypts/Encrypts with AES-256 in Galois/Counter Mode hashed with SHA-384.
#[derive(Debug)]
pub struct Chacha20Poly1305Sha256<CS>(PhantomData<CS>);

impl<CS> Default for Chacha20Poly1305Sha256<CS> {
  fn default() -> Self {
    Self(core::marker::PhantomData)
  }
}

#[cfg(feature = "aws-lc-rs")]
impl CipherSuite for Chacha20Poly1305Sha256<crate::tls::AwsLcRs> {
  type Aead = crate::tls::cipher_suite::Chacha20Poly1305AwsLcRs;
  type Hash = crate::tls::cipher_suite::Sha256AwsLcRs;
  type Hkdf = aws_lc_rs::hkdf::Algorithm;

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Chacha20Poly1305Sha256
  }
}

#[cfg(feature = "rust-crypto")]
impl CipherSuite for Chacha20Poly1305Sha256<crate::tls::RustCrypto> {
  type Aead = chacha20poly1305::ChaCha20Poly1305;
  type Hash = sha2::Sha256;
  type Hkdf = hkdf::Hkdf<Self::Hash>;

  fn ty(&self) -> CipherSuiteTy {
    CipherSuiteTy::Chacha20Poly1305Sha256
  }
}
