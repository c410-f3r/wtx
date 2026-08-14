use crate::{
  codec::{Decode, Encode},
  tls::{
    TlsError, de::De, tls_decode_wrapper::TlsDecodeWrapper, tls_encode_wrapper::TlsEncodeWrapper,
  },
  x509::{KeyTy, SignatureTy},
};

/// Signature Scheme
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SignatureScheme {
  /// `EcdsaSecp256r1Sha256`
  #[default]
  EcdsaSecp256r1Sha256 = 0x0403,
  /// `EcdsaSecp384r1Sha384`
  EcdsaSecp384r1Sha384 = 0x0503,
  /// `RsaPssRsaeSha256`
  RsaPssRsaeSha256 = 0x0804,
  /// `RsaPssRsaeSha384`
  RsaPssRsaeSha384 = 0x0805,
  /// `Ed25519`
  Ed25519 = 0x0807,
  /// `RsaPssPssSha256`
  RsaPssPssSha256 = 0x0809,
  /// `RsaPssPssSha384`
  RsaPssPssSha384 = 0x080a,
}

impl SignatureScheme {
  pub(crate) const PRIORITY: [Self; SignatureScheme::len()] = [
    Self::Ed25519,
    Self::EcdsaSecp256r1Sha256,
    Self::EcdsaSecp384r1Sha384,
    Self::RsaPssPssSha256,
    Self::RsaPssPssSha384,
    Self::RsaPssRsaeSha256,
    Self::RsaPssRsaeSha384,
  ];

  /// Used to verify existing certificates. For example, if a client wants to negotiate only with
  /// [`Self::RsaPssRsaeSha256`] but a server only serves [`SignatureTy::RsaPssSha256`], that would
  /// be a mismatch.
  #[inline]
  pub(crate) const fn cert_kt(self) -> KeyTy {
    match self {
      Self::EcdsaSecp256r1Sha256 => KeyTy::EcdsaP256,
      Self::EcdsaSecp384r1Sha384 => KeyTy::EcdsaP384,
      Self::Ed25519 => KeyTy::Ed25519,
      Self::RsaPssPssSha256 => KeyTy::RsaPssSha256,
      Self::RsaPssPssSha384 => KeyTy::RsaPssSha384,
      Self::RsaPssRsaeSha256 | Self::RsaPssRsaeSha384 => KeyTy::RsaPkcs1,
    }
  }

  /// Used in TLS records like `CertificateVerify`.
  #[inline]
  pub(crate) const fn handshake_st(self) -> SignatureTy {
    match self {
      Self::EcdsaSecp256r1Sha256 => SignatureTy::EcdsaP256,
      Self::EcdsaSecp384r1Sha384 => SignatureTy::EcdsaP384,
      Self::Ed25519 => SignatureTy::Ed25519,
      Self::RsaPssRsaeSha256 | Self::RsaPssPssSha256 => SignatureTy::RsaPssSha256,
      Self::RsaPssRsaeSha384 | Self::RsaPssPssSha384 => SignatureTy::RsaPssSha384,
    }
  }

  pub(crate) const fn len() -> usize {
    7
  }
}

impl<'de> Decode<'de, De> for SignatureScheme {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    Self::try_from(<u16 as Decode<De>>::decode(dw)?)
  }
}

impl Encode<De> for SignatureScheme {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}

impl From<SignatureScheme> for u16 {
  #[inline]
  fn from(value: SignatureScheme) -> Self {
    match value {
      SignatureScheme::EcdsaSecp256r1Sha256 => 0x0403,
      SignatureScheme::EcdsaSecp384r1Sha384 => 0x0503,
      SignatureScheme::RsaPssRsaeSha256 => 0x0804,
      SignatureScheme::RsaPssRsaeSha384 => 0x0805,
      SignatureScheme::Ed25519 => 0x0807,
      SignatureScheme::RsaPssPssSha256 => 0x0809,
      SignatureScheme::RsaPssPssSha384 => 0x080a,
    }
  }
}

impl TryFrom<u16> for SignatureScheme {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u16) -> crate::Result<Self> {
    Ok(match value {
      0x0403 => SignatureScheme::EcdsaSecp256r1Sha256,
      0x0503 => SignatureScheme::EcdsaSecp384r1Sha384,
      0x0804 => SignatureScheme::RsaPssRsaeSha256,
      0x0805 => SignatureScheme::RsaPssRsaeSha384,
      0x0807 => SignatureScheme::Ed25519,
      0x0809 => SignatureScheme::RsaPssPssSha256,
      0x080a => SignatureScheme::RsaPssPssSha384,
      _ => return Err(TlsError::UnknownSignatureScheme.into()),
    })
  }
}
