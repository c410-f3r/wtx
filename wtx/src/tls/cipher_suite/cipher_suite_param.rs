use crate::tls::cipher_suite::CipherSuite;
#[cfg(any(feature = "aws-lc-rs", feature = "rust-crypto"))]
use crate::tls::cipher_suite::{
  CipherSuiteTy, aes_128_gcm_sha_256::Aes128GcmSha256, aes_256_gcm_sha_384::Aes256GcmSha384,
  chacha20_poly1305_sha256::Chacha20Poly1305Sha256,
};

#[cfg(feature = "aws-lc-rs")]
pub(crate) type CipherSuiteParamAwsLcRs = CipherSuiteParam<
  Aes128GcmSha256<crate::tls::AwsLcRs>,
  Aes256GcmSha384<crate::tls::AwsLcRs>,
  Chacha20Poly1305Sha256<crate::tls::AwsLcRs>,
>;
#[cfg(feature = "rust-crypto")]
pub(crate) type CipherSuiteParamRustCrypto = CipherSuiteParam<
  Aes128GcmSha256<crate::tls::RustCrypto>,
  Aes256GcmSha384<crate::tls::RustCrypto>,
  Chacha20Poly1305Sha256<crate::tls::RustCrypto>,
>;
pub(crate) type CipherSuiteParamUnit = CipherSuiteParam<(), (), ()>;

#[derive(Debug)]
pub(crate) enum CipherSuiteParam<A, B, C> {
  Aes128GcmSha256(A),
  Aes256GcmSha384(B),
  Chacha20Poly1305Sha256(C),
}

impl<A, B, C> CipherSuite for CipherSuiteParam<A, B, C>
where
  A: CipherSuite,
  B: CipherSuite,
  C: CipherSuite,
{
  type Aead = crate::tls::cipher_suite::Aes128GcmAwsLcRs;
  type Hash = crate::tls::cipher_suite::Sha256AwsLcRs;
  type Hkdf = ();

  fn ty(&self) -> CipherSuiteTy {
    match self {
      CipherSuiteParam::Aes128GcmSha256(_) => CipherSuiteTy::Aes128GcmSha256,
      CipherSuiteParam::Aes256GcmSha384(_) => CipherSuiteTy::Aes256GcmSha384,
      CipherSuiteParam::Chacha20Poly1305Sha256(_) => CipherSuiteTy::Chacha20Poly1305Sha256,
    }
  }
}

#[cfg(feature = "aws-lc-rs")]
impl From<CipherSuiteTy> for CipherSuiteParamAwsLcRs {
  fn from(value: CipherSuiteTy) -> Self {
    match value {
      CipherSuiteTy::Aes128GcmSha256 => {
        CipherSuiteParamAwsLcRs::Aes128GcmSha256(Aes128GcmSha256::default())
      }
      CipherSuiteTy::Aes256GcmSha384 => {
        CipherSuiteParamAwsLcRs::Aes256GcmSha384(Aes256GcmSha384::default())
      }
      CipherSuiteTy::Chacha20Poly1305Sha256 => {
        CipherSuiteParamAwsLcRs::Chacha20Poly1305Sha256(Chacha20Poly1305Sha256::default())
      }
    }
  }
}

#[cfg(feature = "rust-crypto")]
impl From<CipherSuiteTy> for CipherSuiteParamRustCrypto {
  fn from(value: CipherSuiteTy) -> Self {
    match value {
      CipherSuiteTy::Aes128GcmSha256 => {
        CipherSuiteParamRustCrypto::Aes128GcmSha256(Aes128GcmSha256::default())
      }
      CipherSuiteTy::Aes256GcmSha384 => {
        CipherSuiteParamRustCrypto::Aes256GcmSha384(Aes256GcmSha384::default())
      }
      CipherSuiteTy::Chacha20Poly1305Sha256 => {
        CipherSuiteParamRustCrypto::Chacha20Poly1305Sha256(Chacha20Poly1305Sha256::default())
      }
    }
  }
}
