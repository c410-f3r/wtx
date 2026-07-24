/// Hash Type
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HashTy {
  /// Resulting hash has 32 bytes
  #[default]
  Sha256,
  /// Resulting hash has 48 bytes
  Sha384,
  /// Resulting hash has 64 bytes
  Sha512,
}

impl HashTy {
  #[inline]
  #[cfg(feature = "crypto-alr")]
  pub(crate) const fn rsa_pkcs1_enc_alr(
    self,
  ) -> &'static aws_lc_rs::signature::RsaSignatureEncoding {
    match self {
      HashTy::Sha256 => &aws_lc_rs::signature::RSA_PKCS1_SHA256,
      HashTy::Sha384 => &aws_lc_rs::signature::RSA_PKCS1_SHA384,
      HashTy::Sha512 => &aws_lc_rs::signature::RSA_PKCS1_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-ring")]
  pub(crate) const fn rsa_pkcs1_enc_ring(self) -> &'static dyn ring::signature::RsaEncoding {
    match self {
      HashTy::Sha256 => &ring::signature::RSA_PKCS1_SHA256,
      HashTy::Sha384 => &ring::signature::RSA_PKCS1_SHA384,
      HashTy::Sha512 => &ring::signature::RSA_PKCS1_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-alr")]
  pub(crate) const fn rsa_pkcs1_params_alr(self) -> &'static aws_lc_rs::signature::RsaParameters {
    match self {
      HashTy::Sha256 => &aws_lc_rs::signature::RSA_PKCS1_2048_8192_SHA256,
      HashTy::Sha384 => &aws_lc_rs::signature::RSA_PKCS1_2048_8192_SHA384,
      HashTy::Sha512 => &aws_lc_rs::signature::RSA_PKCS1_2048_8192_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-ring")]
  pub(crate) const fn rsa_pkcs1_params_ring(self) -> &'static ring::signature::RsaParameters {
    match self {
      HashTy::Sha256 => &ring::signature::RSA_PKCS1_2048_8192_SHA256,
      HashTy::Sha384 => &ring::signature::RSA_PKCS1_2048_8192_SHA384,
      HashTy::Sha512 => &ring::signature::RSA_PKCS1_2048_8192_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-alr")]
  pub(crate) const fn rsa_pss_enc_alr(self) -> &'static aws_lc_rs::signature::RsaSignatureEncoding {
    match self {
      HashTy::Sha256 => &aws_lc_rs::signature::RSA_PSS_SHA256,
      HashTy::Sha384 => &aws_lc_rs::signature::RSA_PSS_SHA384,
      HashTy::Sha512 => &aws_lc_rs::signature::RSA_PSS_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-ring")]
  pub(crate) const fn rsa_pss_enc_ring(self) -> &'static dyn ring::signature::RsaEncoding {
    match self {
      HashTy::Sha256 => &ring::signature::RSA_PSS_SHA256,
      HashTy::Sha384 => &ring::signature::RSA_PSS_SHA384,
      HashTy::Sha512 => &ring::signature::RSA_PSS_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-alr")]
  pub(crate) const fn rsa_pss_params_alr(self) -> &'static aws_lc_rs::signature::RsaParameters {
    match self {
      HashTy::Sha256 => &aws_lc_rs::signature::RSA_PSS_2048_8192_SHA256,
      HashTy::Sha384 => &aws_lc_rs::signature::RSA_PSS_2048_8192_SHA384,
      HashTy::Sha512 => &aws_lc_rs::signature::RSA_PSS_2048_8192_SHA512,
    }
  }

  #[inline]
  #[cfg(feature = "crypto-ring")]
  pub(crate) const fn rsa_pss_params_ring(self) -> &'static ring::signature::RsaParameters {
    match self {
      HashTy::Sha256 => &ring::signature::RSA_PSS_2048_8192_SHA256,
      HashTy::Sha384 => &ring::signature::RSA_PSS_2048_8192_SHA384,
      HashTy::Sha512 => &ring::signature::RSA_PSS_2048_8192_SHA512,
    }
  }
}
