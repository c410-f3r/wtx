use crate::{
  crypto::{
    Aes128GcmAwsLcRs, Aes256GcmAwsLcRs, Chacha20Poly1305AwsLcRs, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, write_tag},
  },
  rng::CryptoRng,
};
use aws_lc_rs::aead::{
  AES_128_GCM, AES_256_GCM, Aad, Algorithm, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey,
};

impl Aead for Aes128GcmAwsLcRs {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    local_decrypt(
      &AES_128_GCM,
      associated_data,
      data,
      CryptoError::InvalidAes128GcmData,
      nonce,
      secret,
    )
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
    local_encrypt_vectored_data(
      &AES_128_GCM,
      associated_data,
      CryptoError::InvalidAes128GcmData,
      nonce,
      plaintext,
      rng,
      secret,
      tag,
    )
  }
}

impl Aead for Aes256GcmAwsLcRs {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    local_decrypt(
      &AES_256_GCM,
      associated_data,
      data,
      CryptoError::InvalidAes256GcmData,
      nonce,
      secret,
    )
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
    local_encrypt_vectored_data(
      &AES_256_GCM,
      associated_data,
      CryptoError::InvalidAes256GcmData,
      nonce,
      plaintext,
      rng,
      secret,
      tag,
    )
  }
}

impl Aead for Chacha20Poly1305AwsLcRs {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    local_decrypt(
      &CHACHA20_POLY1305,
      associated_data,
      data,
      CryptoError::InvalidChacha20Poly1305Data,
      nonce,
      secret,
    )
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
    local_encrypt_vectored_data(
      &CHACHA20_POLY1305,
      associated_data,
      CryptoError::InvalidChacha20Poly1305Data,
      nonce,
      plaintext,
      rng,
      secret,
      tag,
    )
  }
}

common_aead_functions!();
