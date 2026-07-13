use crate::crypto::{
  Aead,
  aead::{AEAD_NONCE_LEN, AEAD_TAG_LEN},
};

type Aes128GcmTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Aes128GcmRing,
  feature = "crypto-graviola" => crate::crypto::Aes128GcmGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Aes128GcmAwsLcRs,
  _ => crate::crypto::AeadDummy::<[u8; 16]>
};
type Aes256GcmTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Aes256GcmRing,
  feature = "crypto-graviola" => crate::crypto::Aes256GcmGraviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Aes256GcmAwsLcRs,
  _ => crate::crypto::AeadDummy::<[u8; 32]>
};
type Chacha20Poly1305Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Chacha20Poly1305Ring,
  feature = "crypto-graviola" => crate::crypto::Chacha20Poly1305Graviola,
  feature = "crypto-aws-lc-rs" => crate::crypto::Chacha20Poly1305AwsLcRs,
  _ => crate::crypto::AeadDummy::<[u8; 32]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Copy, Debug)]
pub struct Aes128GcmGlobal;

impl Aead for Aes128GcmGlobal {
  type Secret = <Aes128GcmTy as Aead>::Secret;

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    Aes128GcmTy::decrypt_parts(associated_data, data, nonce, secret)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    Aes128GcmTy::encrypt_parts(associated_data, nonce, plaintext, secret)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Clone, Copy, Debug)]
pub struct Aes256GcmGlobal;

impl Aead for Aes256GcmGlobal {
  type Secret = <Aes256GcmTy as Aead>::Secret;

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    Aes256GcmTy::decrypt_parts(associated_data, data, nonce, secret)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    Aes256GcmTy::encrypt_parts(associated_data, nonce, plaintext, secret)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct Chacha20Poly1305Global;

impl Aead for Chacha20Poly1305Global {
  type Secret = <Chacha20Poly1305Ty as Aead>::Secret;

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    Chacha20Poly1305Ty::decrypt_parts(associated_data, data, nonce, secret)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    Chacha20Poly1305Ty::encrypt_parts(associated_data, nonce, plaintext, secret)
  }
}
