use crate::{crypto::{AeadStub, HashStub}, tls::cipher_suite::CipherSuite};
#[cfg(any(feature = "tls-aws-lc-rs", feature = "tls-rust-crypto"))]
use crate::tls::{
  protocol::{
    aes_128_gcm_sha_256::Aes128GcmSha256,
    aes_256_gcm_sha_384::Aes256GcmSha384,
    chacha20_poly1305_sha256::Chacha20Poly1305Sha256,
  },
};
use crate::tls::CipherSuiteTy;

#[cfg(feature = "tls-aws-lc-rs")]
pub(crate) type CipherSuiteParamAwsLcRs = CipherSuiteParam<
  Aes128GcmSha256<crate::tls::AwsLcRs>,
  Aes256GcmSha384<crate::tls::AwsLcRs>,
  Chacha20Poly1305Sha256<crate::tls::AwsLcRs>,
>;
#[cfg(feature = "tls-rust-crypto")]
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
  type Aead = AeadStub<()>;
  type Hash = HashStub<[u8; 0]>;

  fn aead(&self) -> &Self::Aead {
    todo!()
  }

  fn hash(&self) -> &Self::Hash {
    todo!()
  }

  fn ty(&self) -> CipherSuiteTy {
    match self {
      CipherSuiteParam::Aes128GcmSha256(_) => CipherSuiteTy::Aes128GcmSha256,
      CipherSuiteParam::Aes256GcmSha384(_) => CipherSuiteTy::Aes256GcmSha384,
      CipherSuiteParam::Chacha20Poly1305Sha256(_) => CipherSuiteTy::Chacha20Poly1305Sha256,
    }
  }
}
