use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::de::De,
};

create_enum! {
  /// The algorithm used in digital signatures.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum SignatureScheme<u16> {
    RsaPkcs1Sha256 = (0x0401),
    RsaPkcs1Sha384 = (0x0501),
    RsaPkcs1Sha512 = (0x0601),

    EcdsaSecp256r1Sha256 = (0x0403),
    EcdsaSecp384r1Sha384 = (0x0503),

    RsaPssRsaeSha256 = (0x0804),
    RsaPssRsaeSha384 = (0x0805),
    RsaPssRsaeSha512 = (0x0806),

    Ed25519 = (0x0807),
  }
}

impl SignatureScheme {
  pub(crate) const PRIORITY: [SignatureScheme; Self::len()] = [
    Self::EcdsaSecp256r1Sha256,
    Self::RsaPssRsaeSha256,
    Self::RsaPkcs1Sha256,
    Self::EcdsaSecp384r1Sha384,
    Self::RsaPssRsaeSha384,
    Self::Ed25519,
    Self::RsaPkcs1Sha384,
    Self::RsaPssRsaeSha512,
    Self::RsaPkcs1Sha512,
  ];
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

//#[cfg(feature = "rustls-webpki")]
//impl TryFrom<SignatureScheme> for &'static dyn rustls_pki_types::SignatureVerificationAlgorithm {
//  type Error = crate::Error;
//
//  #[inline]
//  fn try_from(from: SignatureScheme) -> Result<Self, Self::Error> {
//    return Ok(match from {
//      SignatureScheme::RsaPkcs1Sha256 => webpki::ring::RSA_PKCS1_2048_8192_SHA256,
//      SignatureScheme::RsaPkcs1Sha384 => webpki::ring::RSA_PKCS1_2048_8192_SHA384,
//      SignatureScheme::RsaPkcs1Sha512 => webpki::ring::RSA_PKCS1_2048_8192_SHA512,
//
//      SignatureScheme::EcdsaSecp256r1Sha256 => webpki::ring::ECDSA_P256_SHA256,
//      SignatureScheme::EcdsaSecp384r1Sha384 => webpki::ring::ECDSA_P384_SHA384,
//
//      SignatureScheme::RsaPssRsaeSha256 => webpki::ring::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
//      SignatureScheme::RsaPssRsaeSha384 => webpki::ring::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
//      SignatureScheme::RsaPssRsaeSha512 => webpki::ring::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
//
//      SignatureScheme::Ed25519 => webpki::ring::ED25519,
//
//      _ => return Err(TlsError::UnknownWebpkiSignatureScheme.into()),
//    });
//  }
//}

impl<'de> Decode<'de, De> for SignatureScheme {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    Ok(Self::try_from(<u16 as Decode<De>>::decode(dw)?)?)
  }
}

impl Encode<De> for SignatureScheme {
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
