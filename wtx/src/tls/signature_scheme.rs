use crate::tls::TlsError;

create_enum! {
  /// The signature algorithm used in digital signatures.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum SignatureScheme<u16> {
    RsaPkcs1Sha256 = (0x0401),
    RsaPkcs1Sha384 = (0x0501),
    RsaPkcs1Sha512 = (0x0601),

    EcdsaSecp256r1Sha256 = (0x0403),
    EcdsaSecp384r1Sha384 = (0x0503),
    EcdsaSecp521r1Sha512 = (0x0603),

    RsaPssRsaeSha256 = (0x0804),
    RsaPssRsaeSha384 = (0x0805),
    RsaPssRsaeSha512 = (0x0806),

    Ed25519 = (0x0807),
    Ed448 = (0x0808),
  }
}

impl SignatureScheme {
  //  fn ring(&self) {
  //    &[
  //      (SignatureScheme::RsaPkcs1Sha256, &[webpki::ring::RSA_PKCS1_2048_8192_SHA256]),
  //      (SignatureScheme::RsaPkcs1Sha384, &[webpki::ring::RSA_PKCS1_2048_8192_SHA384]),
  //      (SignatureScheme::RsaPkcs1Sha512, &[webpki::ring::RSA_PKCS1_2048_8192_SHA512]),
  //      (
  //        SignatureScheme::EcdsaSecp256r1Sha256,
  //        &[
  //          webpki::ring::ECDSA_P256_SHA256,
  //          webpki::ring::ECDSA_P384_SHA256,
  //          webpki::ring::ECDSA_P521_SHA256,
  //        ],
  //      ),
  //      (
  //        SignatureScheme::EcdsaSecp384r1Sha384,
  //        &[
  //          webpki::ring::ECDSA_P384_SHA384,
  //          webpki::ring::ECDSA_P256_SHA384,
  //          webpki::ring::ECDSA_P521_SHA384,
  //        ],
  //      ),
  //      (SignatureScheme::EcdsaSecp521r1Sha512, &[webpki::ring::ECDSA_P521_SHA512]),
  //      (SignatureScheme::RsaPssRsaeSha256, &[webpki::ring::RSA_PSS_2048_8192_SHA256_LEGACY_KEY]),
  //      (SignatureScheme::RsaPssRsaeSha384, &[webpki::ring::RSA_PSS_2048_8192_SHA384_LEGACY_KEY]),
  //      (SignatureScheme::RsaPssRsaeSha512, &[webpki::ring::RSA_PSS_2048_8192_SHA512_LEGACY_KEY]),
  //      (SignatureScheme::Ed25519, &[webpki::ring::ED25519]),
  //    ];
  //  }
}

#[cfg(feature = "tls-ring")]
impl TryInto<&'static dyn rustls_pki_types::SignatureVerificationAlgorithm> for SignatureScheme {
  type Error = crate::Error;

  #[inline]
  fn try_into(
    self,
  ) -> Result<&'static dyn rustls_pki_types::SignatureVerificationAlgorithm, Self::Error> {
    return Ok(match self {
      Self::RsaPkcs1Sha256 => webpki::ring::RSA_PKCS1_2048_8192_SHA256,
      Self::RsaPkcs1Sha384 => webpki::ring::RSA_PKCS1_2048_8192_SHA384,
      Self::RsaPkcs1Sha512 => webpki::ring::RSA_PKCS1_2048_8192_SHA512,

      Self::EcdsaSecp256r1Sha256 => webpki::ring::ECDSA_P256_SHA256,
      Self::EcdsaSecp384r1Sha384 => webpki::ring::ECDSA_P384_SHA384,

      Self::RsaPssRsaeSha256 => webpki::ring::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
      Self::RsaPssRsaeSha384 => webpki::ring::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
      Self::RsaPssRsaeSha512 => webpki::ring::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,

      Self::Ed25519 => webpki::ring::ED25519,

      _ => return Err(TlsError::UnknownWebpkiSignatureScheme.into()),
    });
  }
}
