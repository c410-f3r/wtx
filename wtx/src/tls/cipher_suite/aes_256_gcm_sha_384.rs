use core::marker::PhantomData;

use crate::tls::cipher_suite::{
  Aes256GcmAwsLc, Aes256GcmRing, CipherSuite, CipherSuiteTy, Sha384AwsLc, Sha384Ring,
};

#[derive(Debug)]
pub struct Aes256GcmSha384<CS>(PhantomData<CS>);

#[cfg(feature = "tls-aws-lc")]
impl CipherSuite for Aes256GcmSha384<crate::tls::AwsLc> {
  const TY: CipherSuiteTy = CipherSuiteTy::TlsAes256GcmSha384;

  type Aead = Aes256GcmAwsLc;
  type Hash = Sha384AwsLc;
}

#[cfg(feature = "tls-ring")]
impl CipherSuite for Aes256GcmSha384<crate::tls::Ring> {
  const TY: CipherSuiteTy = CipherSuiteTy::TlsAes256GcmSha384;

  type Aead = Aes256GcmRing;
  type Hash = Sha384Ring;
}

#[cfg(feature = "tls-rust-crypto")]
impl CipherSuite for Aes256GcmSha384<crate::tls::RustCrypto> {
  const TY: CipherSuiteTy = CipherSuiteTy::TlsAes256GcmSha384;

  type Aead = aes_gcm::Aes256Gcm;
  type Hash = sha2::Sha384;
}
