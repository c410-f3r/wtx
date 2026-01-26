use crate::{
  de::{Decode, Encode},
  tls::{de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
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
