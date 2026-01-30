
use crate::{
  crypto::{
    Aes128GcmAwsLcRs, Aes256GcmAwsLcRs, Chacha20Poly1305AwsLcRs, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content, write_tag},
  },
  rng::CryptoRng,
};
use aws_lc_rs::aead::{
  AES_128_GCM, AES_256_GCM, Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey,
};

impl Aead<16> for Aes128GcmAwsLcRs {
  #[inline]
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 16],
  ) -> crate::Result<&'encrypted mut [u8]> {
    local_decrypt(
      &AES_128_GCM,
      associated_data,
      encrypted_data,
      CryptoError::InvalidAes128GcmData,
      secret,
    )
  }

  #[inline]
  fn encrypt_vectored_data<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &[u8; 16],
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

impl Aead<32> for Aes256GcmAwsLcRs {
  #[inline]
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]> {
    local_decrypt(
      &AES_256_GCM,
      associated_data,
      encrypted_data,
      CryptoError::InvalidAes256GcmData,
      secret,
    )
  }

  #[inline]
  fn encrypt_vectored_data<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &[u8; 32],
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

impl Aead<32> for Chacha20Poly1305AwsLcRs {
  #[inline]
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]> {
    local_decrypt(
      &CHACHA20_POLY1305,
      associated_data,
      encrypted_data,
      CryptoError::InvalidChacha20Poly1305Data,
      secret,
    )
  }

  #[inline]
  fn encrypt_vectored_data<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &[u8; 32],
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

#[inline]
fn local_decrypt<'encrypted, const S: usize>(
  algorithm: &aws_lc_rs::aead::Algorithm,
  associated_data: &[u8],
  encrypted_data: &'encrypted mut [u8],
  error: CryptoError,
  secret: &[u8; S],
) -> crate::Result<&'encrypted mut [u8]> {
  let (nonce, content) = split_nonce_content(encrypted_data, error)?;
  let bytes = LessSafeKey::new(UnboundKey::new(algorithm, secret).map_err(|_| error)?)
    .open_in_place(Nonce::assume_unique_for_key(nonce), Aad::from(associated_data), content)
    .map_err(|_| error)?;
  Ok(bytes)
}

#[inline]
fn local_encrypt_vectored_data<RNG, const S: usize>(
  algorithm: &aws_lc_rs::aead::Algorithm,
  associated_data: &[u8],
  error: CryptoError,
  nonce: [&mut u8; NONCE_LEN],
  plaintext: &mut [u8],
  rng: &mut RNG,
  secret: &[u8; S],
  tag: [&mut u8; TAG_LEN],
) -> crate::Result<()>
where
  RNG: CryptoRng,
{
  let local_tag = LessSafeKey::new(UnboundKey::new(algorithm, secret).map_err(|_| error)?)
    .seal_in_place_separate_tag(
      Nonce::assume_unique_for_key(generate_nonce(nonce, rng)),
      Aad::from(associated_data),
      plaintext,
    )
    .map_err(|_| error)?
    .as_ref()
    .try_into()?;
  write_tag(local_tag, tag);
  Ok(())
}
