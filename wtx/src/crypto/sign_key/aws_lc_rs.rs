use crate::crypto::{
  CryptoError, Ed25519SignKeyAwsLcRs, P256SignKeyAwsLcRs, P384SignKeyAwsLcRs,
  RsaPssSignKeySha256AwsLcRs, RsaPssSignKeySha384AwsLcRs, sign_key::SignKey,
};
use aws_lc_rs::signature::{
  ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P384_SHA384_ASN1_SIGNING, EcdsaKeyPair, Ed25519KeyPair,
  RsaKeyPair,
};

impl SignKey for Ed25519SignKeyAwsLcRs {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(
      Ed25519KeyPair::from_pkcs8_maybe_unchecked(bytes)
        .map_err(|_err| CryptoError::SignKeyError)?,
    ))
  }
}

impl SignKey for P256SignKeyAwsLcRs {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(
      EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, bytes)
        .map_err(|_err| CryptoError::SignKeyError)?,
    ))
  }
}

impl SignKey for P384SignKeyAwsLcRs {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(
      EcdsaKeyPair::from_pkcs8(&ECDSA_P384_SHA384_ASN1_SIGNING, bytes)
        .map_err(|_err| CryptoError::SignKeyError)?,
    ))
  }
}

impl SignKey for RsaPssSignKeySha256AwsLcRs {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(RsaKeyPair::from_pkcs8(bytes).map_err(|_err| CryptoError::SignKeyError)?))
  }
}

impl SignKey for RsaPssSignKeySha384AwsLcRs {
  #[inline]
  fn from_pkcs8(bytes: &[u8]) -> crate::Result<Self> {
    Ok(Self(RsaKeyPair::from_pkcs8(bytes).map_err(|_err| CryptoError::SignKeyError)?))
  }
}
