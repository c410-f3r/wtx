use crate::{
  crypto::{
    Aead,
    aead::{NONCE_LEN, TAG_LEN},
  },
  rng::CryptoRng,
};

type Aes128GcmTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Aes128GcmRing,
  feature = "crypto-graviola" => crate::crypto::Aes128GcmGraviola,
  feature = "crypto-rust-crypto" => crate::crypto::Aes128GcmRustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Aes128GcmAwsLcRs,
  _ => crate::crypto::AeadStub::<[u8; 16]>
};
type Aes256GcmTy = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Aes256GcmRing,
  feature = "crypto-graviola" => crate::crypto::Aes256GcmGraviola,
  feature = "crypto-rust-crypto" => crate::crypto::Aes256GcmRustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Aes256GcmAwsLcRs,
  _ => crate::crypto::AeadStub::<[u8; 32]>
};
type Chacha20Poly1305Ty = cfg_select! {
  feature = "crypto-ring" => crate::crypto::Chacha20Poly1305Ring,
  feature = "crypto-graviola" => crate::crypto::Chacha20Poly1305Graviola,
  feature = "crypto-rust-crypto" => crate::crypto::Chacha20Poly1305RustCrypto,
  feature = "crypto-aws-lc-rs" => crate::crypto::Chacha20Poly1305AwsLcRs,
  _ => crate::crypto::AeadStub::<[u8; 32]>
};

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalAes128GcmTy;

impl Aead for GlobalAes128GcmTy {
  type Secret = <Aes128GcmTy as Aead>::Secret;

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    Aes128GcmTy::decrypt_in_place(associated_data, encrypted_data, secret)
  }

  #[inline]
  fn encrypt_parts<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &Self::Secret,
    tag: [&mut u8; TAG_LEN],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    Aes128GcmTy::encrypt_parts(associated_data, nonce, plaintext, rng, secret, tag)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalAes256GcmTy;

impl Aead for GlobalAes256GcmTy {
  type Secret = <Aes256GcmTy as Aead>::Secret;

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    Aes256GcmTy::decrypt_in_place(associated_data, encrypted_data, secret)
  }

  #[inline]
  fn encrypt_parts<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &Self::Secret,
    tag: [&mut u8; TAG_LEN],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    Aes256GcmTy::encrypt_parts(associated_data, nonce, plaintext, rng, secret, tag)
  }
}

/// A structure that delegates execution to the selected crypto backend.
#[derive(Debug)]
pub struct GlobalChacha20Poly1305Ty;

impl Aead for GlobalChacha20Poly1305Ty {
  type Secret = <Chacha20Poly1305Ty as Aead>::Secret;

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    Chacha20Poly1305Ty::decrypt_in_place(associated_data, encrypted_data, secret)
  }

  #[inline]
  fn encrypt_parts<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &Self::Secret,
    tag: [&mut u8; TAG_LEN],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    Chacha20Poly1305Ty::encrypt_parts(associated_data, nonce, plaintext, rng, secret, tag)
  }
}
