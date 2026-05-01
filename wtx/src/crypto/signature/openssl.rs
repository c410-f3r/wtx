use crate::{
  collection::Vector,
  crypto::{
    CryptoError, Ed25519Openssl, Ed25519SignKeyOpenssl, P256Openssl, P256SignKeyOpenssl,
    P384Openssl, P384SignKeyOpenssl, RsaPssRsaeSha256Openssl, RsaPssRsaeSha384Openssl,
    RsaPssSignKeySha256Openssl, RsaPssSignKeySha384Openssl, Signature,
  },
  rng::CryptoRng,
};
use openssl::{
  ec::{EcGroup, EcKey},
  hash::MessageDigest,
  nid::Nid,
  pkey::{PKey, Public},
  rsa::Rsa,
  sign::{RsaPssSaltlen, Signer, Verifier},
};

impl Signature for P256Openssl {
  type SignKey = P256SignKeyOpenssl;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signer = Signer::new(MessageDigest::sha256(), &sign_key.0)?;
    signer.update(msg)?;
    let mut rslt = [0; 64];
    let _ = signer.sign(&mut rslt)?;
    Ok(rslt)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let pkey = ec_public_key_from_uncompressed(Nid::X9_62_PRIME256V1, pk)?;
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey)?;
    verifier.update(msg)?;
    if !verifier.verify(signature)? {
      return Err(CryptoError::SignatureError.into());
    }
    Ok(())
  }
}

impl Signature for P384Openssl {
  type SignKey = P384SignKeyOpenssl;
  type SignOutput = [u8; 96];

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signer = Signer::new(MessageDigest::sha384(), &sign_key.0)?;
    signer.update(msg)?;
    let mut rslt = [0; 96];
    let _ = signer.sign(&mut rslt)?;
    Ok(rslt)
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let pkey = ec_public_key_from_uncompressed(Nid::SECP384R1, pk)?;
    let mut verifier = Verifier::new(MessageDigest::sha384(), &pkey)?;
    verifier.update(msg)?;
    if !verifier.verify(signature)? {
      return Err(CryptoError::SignatureError.into());
    }
    Ok(())
  }
}

impl Signature for Ed25519Openssl {
  type SignKey = Ed25519SignKeyOpenssl;
  type SignOutput = [u8; 64];

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signer = Signer::new_without_digest(&sign_key.0)?;
    signer.update(msg)?;
    signer.sign_to_vec()?.try_into().map_err(|_err| CryptoError::SignatureError.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let pkey = PKey::public_key_from_raw_bytes(pk, openssl::pkey::Id::ED25519)?;
    let mut verifier = Verifier::new_without_digest(&pkey)?;
    verifier.update(msg)?;
    if !verifier.verify(signature)? {
      return Err(CryptoError::SignatureError.into());
    }
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha256Openssl {
  type SignKey = RsaPssSignKeySha256Openssl;
  type SignOutput = Vector<u8>;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signer = Signer::new(MessageDigest::sha256(), &sign_key.0)?;
    signer.set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)?;
    signer.set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)?;
    signer.update(msg)?;
    Ok(signer.sign_to_vec()?.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let rsa = Rsa::public_key_from_der_pkcs1(pk)?;
    let pkey = PKey::from_rsa(rsa)?;
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey)?;
    verifier.set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)?;
    verifier.set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)?;
    verifier.update(msg)?;
    if !verifier.verify(signature)? {
      return Err(CryptoError::SignatureError.into());
    }
    Ok(())
  }
}

impl Signature for RsaPssRsaeSha384Openssl {
  type SignKey = RsaPssSignKeySha384Openssl;
  type SignOutput = Vector<u8>;

  #[inline]
  fn sign<RNG>(
    _: &mut RNG,
    sign_key: &mut Self::SignKey,
    msg: &[u8],
  ) -> crate::Result<Self::SignOutput>
  where
    RNG: CryptoRng,
  {
    let mut signer = Signer::new(MessageDigest::sha384(), &sign_key.0)?;
    signer.set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)?;
    signer.set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)?;
    signer.update(msg)?;
    Ok(signer.sign_to_vec()?.into())
  }

  #[inline]
  fn validate(pk: &[u8], msg: &[u8], signature: &[u8]) -> crate::Result<()> {
    let rsa = Rsa::public_key_from_der_pkcs1(pk)?;
    let pkey = PKey::from_rsa(rsa)?;
    let mut verifier = Verifier::new(MessageDigest::sha384(), &pkey)?;
    verifier.set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)?;
    verifier.set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)?;
    verifier.update(msg)?;
    verifier.verify(signature)?.then_some(()).ok_or(CryptoError::SignatureError.into())
  }
}

fn ec_public_key_from_uncompressed(nid: Nid, pk: &[u8]) -> crate::Result<PKey<Public>> {
  let group = EcGroup::from_curve_name(nid)?;
  let mut ctx = openssl::bn::BigNumContext::new()?;
  let point = openssl::ec::EcPoint::from_bytes(&group, pk, &mut ctx)?;
  Ok(PKey::from_ec_key(EcKey::from_public_key(&group, &point)?)?)
}
