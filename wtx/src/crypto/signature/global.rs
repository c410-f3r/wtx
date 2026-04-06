use crate::crypto::Signature;

type P256Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P256Ring,
  feature = "crypto-graviola" => crate::crypto::P256Graviola,
  feature = "crypto-rust-crypto" => crate::crypto::P256RustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::P256AwsLcRs,
  _ => crate::crypto::SignatureStub::<(), [u8; 64]>
};
type P384Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::P384Ring,
  feature = "crypto-graviola" => crate::crypto::P384Graviola,
  feature = "crypto-rust-crypto" => crate::crypto::P384RustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::P384AwsLcRs,
  _ => crate::crypto::SignatureStub::<(), [u8; 96]>
};
type Ed25519Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Ed25519Ring,
  feature = "crypto-graviola" => crate::crypto::Ed25519Graviola,
  feature = "crypto-rust-crypto" => crate::crypto::Ed25519RustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Ed25519AwsLcRs,
  _ => crate::crypto::SignatureStub::<(), [u8; 64]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalP256Signature;

impl Signature for GlobalP256Signature {
  type SignKey = <P256Ty as Signature>::SignKey;
  type SignOutput = <P256Ty as Signature>::SignOutput;

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    P256Ty::sign(sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    P256Ty::validate(pk, msg, signature)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalP384Signature;

impl Signature for GlobalP384Signature {
  type SignKey = <P384Ty as Signature>::SignKey;
  type SignOutput = <P384Ty as Signature>::SignOutput;

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    P384Ty::sign(sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    P384Ty::validate(pk, msg, signature)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalEd25519;

impl Signature for GlobalEd25519 {
  type SignKey = <Ed25519Ty as Signature>::SignKey;
  type SignOutput = <Ed25519Ty as Signature>::SignOutput;

  #[inline]
  fn sign(sign_key: &mut Self::SignKey, msg: &[u8]) -> crate::Result<Self::SignOutput> {
    Ed25519Ty::sign(sign_key, msg)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    Ed25519Ty::validate(pk, msg, signature)
  }
}
