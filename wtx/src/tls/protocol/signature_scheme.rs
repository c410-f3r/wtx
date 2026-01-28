use crate::{
  de::{Decode, Encode},
  tls::{de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

create_enum! {
  /// The algorithms used in digital signatures.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum SignatureScheme<u16> {
    RsaPkcs1Sha256 = (0x0401),
    RsaPkcs1Sha384 = (0x0501),

    EcdsaSecp256r1Sha256 = (0x0403),
    EcdsaSecp384r1Sha384 = (0x0503),

    RsaPssRsaeSha256 = (0x0804),
    RsaPssRsaeSha384 = (0x0805),

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
  ];

  pub(crate) const fn is_rsa(self) -> bool {
    matches!(
      self,
      SignatureScheme::RsaPkcs1Sha256
        | SignatureScheme::RsaPkcs1Sha384
        | SignatureScheme::RsaPssRsaeSha256
        | SignatureScheme::RsaPssRsaeSha384
    )
  }
}

impl<'de> Decode<'de, De> for SignatureScheme {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self::try_from(<u16 as Decode<De>>::decode(dw)?)?)
  }
}

impl Encode<De> for SignatureScheme {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}

//impl rustls_pki_types::SignatureVerificationAlgorithm for SignatureScheme {
//  fn public_key_alg_id(&self) -> rustls_pki_types::AlgorithmIdentifier {
//    match self {
//      SignatureScheme::RsaPkcs1Sha256 => rustls_pki_types::alg_id::RSA_PKCS1_SHA256,
//      SignatureScheme::RsaPkcs1Sha384 => rustls_pki_types::alg_id::RSA_PKCS1_SHA384,
//      SignatureScheme::EcdsaSecp256r1Sha256 => rustls_pki_types::alg_id::ECDSA_SHA256,
//      SignatureScheme::EcdsaSecp384r1Sha384 => rustls_pki_types::alg_id::ECDSA_SHA384,
//      SignatureScheme::RsaPssRsaeSha256 => rustls_pki_types::alg_id::RSA_PSS_SHA256,
//      SignatureScheme::RsaPssRsaeSha384 => rustls_pki_types::alg_id::RSA_PSS_SHA384,
//      SignatureScheme::Ed25519 => rustls_pki_types::alg_id::ED25519,
//    }
//  }
//
//  fn signature_alg_id(&self) -> rustls_pki_types::AlgorithmIdentifier {
//    match self {
//      SignatureScheme::RsaPkcs1Sha256 => rustls_pki_types::alg_id::RSA_PKCS1_SHA256,
//      SignatureScheme::RsaPkcs1Sha384 => rustls_pki_types::alg_id::RSA_PKCS1_SHA384,
//      SignatureScheme::EcdsaSecp256r1Sha256 => rustls_pki_types::alg_id::ECDSA_SHA256,
//      SignatureScheme::EcdsaSecp384r1Sha384 => rustls_pki_types::alg_id::ECDSA_SHA384,
//      SignatureScheme::RsaPssRsaeSha256 => rustls_pki_types::alg_id::RSA_PSS_SHA256,
//      SignatureScheme::RsaPssRsaeSha384 => rustls_pki_types::alg_id::RSA_PSS_SHA384,
//      SignatureScheme::Ed25519 => rustls_pki_types::alg_id::ED25519,
//    }
//  }
//
//  fn verify_signature(
//    &self,
//    public_key: &[u8],
//    message: &[u8],
//    signature: &[u8],
//  ) -> Result<(), rustls_pki_types::InvalidSignature> {
//    signature::UnparsedPublicKey::new(self.verification_alg, public_key)
//      .verify(message, signature)
//      .map_err(|_| InvalidSignature)
//  }
//}
