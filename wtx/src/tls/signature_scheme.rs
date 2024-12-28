/// The signature algorithm used in digital signatures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureScheme {
  /* RSASSA-PKCS1-v1_5 algorithms */
  RsaPkcs1Sha256,
  RsaPkcs1Sha384,
  RsaPkcs1Sha512,

  /* ECDSA algorithms */
  EcdsaSecp256r1Sha256,
  EcdsaSecp384r1Sha384,
  EcdsaSecp521r1Sha512,

  /* RSASSA-PSS algorithms with public key OID rsaEncryption */
  RsaPssRsaeSha256,
  RsaPssRsaeSha384,
  RsaPssRsaeSha512,

  /* EdDSA algorithms */
  Ed25519,
  Ed448,

  /* RSASSA-PSS algorithms with public key OID RSASSA-PSS */
  RsaPssPssSha256,
  RsaPssPssSha384,
  RsaPssPssSha512,

  Sha224Ecdsa,
  Sha224Rsa,
  Sha224Dsa,
}
