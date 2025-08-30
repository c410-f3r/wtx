use crate::tls::cipher_suite::{
  Aes128GcmAwsLc, Aes128GcmRing, CipherSuite, CipherSuiteTy, Sha256AwsLc, Sha256Ring,
};
use core::marker::PhantomData;

#[derive(Debug)]
pub struct Aes128GcmSha256<CS>(PhantomData<CS>);

#[cfg(feature = "aws-lc-rs")]
impl CipherSuite for Aes128GcmSha256<crate::tls::AwsLc> {
  const TY: CipherSuiteTy = CipherSuiteTy::TlsAes128GcmSha256;

  type Aead = Aes128GcmAwsLc;
  type Hash = Sha256AwsLc;
}

#[cfg(feature = "ring")]
impl CipherSuite for Aes128GcmSha256<crate::tls::Ring> {
  const TY: CipherSuiteTy = CipherSuiteTy::TlsAes128GcmSha256;

  type Aead = Aes128GcmRing;
  type Hash = Sha256Ring;
}

#[cfg(feature = "rust-crypto")]
impl CipherSuite for Aes128GcmSha256<crate::tls::RustCrypto> {
  const TY: CipherSuiteTy = CipherSuiteTy::TlsAes128GcmSha256;

  type Aead = aes_gcm::Aes128Gcm;
  type Hash = sha2::Sha256;
}
