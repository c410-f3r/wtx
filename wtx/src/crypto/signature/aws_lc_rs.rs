use crate::crypto::{CryptoError, Ed25519AwsLcRs, P256AwsLcRs, P384AwsLcRs, Signature};
use aws_lc_rs::{
  rand::SystemRandom,
  signature::{
    ECDSA_P256_SHA256_FIXED, ECDSA_P384_SHA384_FIXED, ED25519, EcdsaKeyPair, Ed25519KeyPair,
    UnparsedPublicKey,
  },
};

impl Signature for P256AwsLcRs {
  type SignKey = EcdsaKeyPair;
  type SignOutput = aws_lc_rs::signature::Signature;

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    sign_key.sign(&SystemRandom::new(), msg).map_err(|_err| CryptoError::SignatureError.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ok(
      UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, pk)
        .verify(msg, signature)
        .map_err(|_err| CryptoError::SignatureError)?,
    )
  }
}

impl Signature for P384AwsLcRs {
  type SignKey = EcdsaKeyPair;
  type SignOutput = aws_lc_rs::signature::Signature;

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    sign_key.sign(&SystemRandom::new(), msg).map_err(|_err| CryptoError::SignatureError.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ok(
      UnparsedPublicKey::new(&ECDSA_P384_SHA384_FIXED, pk)
        .verify(msg, signature)
        .map_err(|_err| CryptoError::SignatureError)?,
    )
  }
}

impl Signature for Ed25519AwsLcRs {
  type SignKey = Ed25519KeyPair;
  type SignOutput = aws_lc_rs::signature::Signature;

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    Ok(sign_key.sign(msg))
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ok(
      UnparsedPublicKey::new(&ED25519, pk)
        .verify(msg, signature)
        .map_err(|_err| CryptoError::SignatureError)?,
    )
  }
}
