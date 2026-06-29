use crate::{
  crypto::{
    Aes128GcmOpenssl, Aes256GcmOpenssl, Chacha20Poly1305Openssl, CryptoError,
    aead::{AEAD_NONCE_LEN, AEAD_TAG_LEN, Aead, split_content_tag},
  },
  misc::SensitiveBytes,
};
use openssl::symm::{Cipher, decrypt_aead, encrypt_aead};

impl Aead for Aes128GcmOpenssl {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    local_decrypt(
      associated_data,
      Cipher::aes_128_gcm(),
      data,
      CryptoError::InvalidAes128GcmData,
      nonce,
      secret,
    )
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    local_encrypt(associated_data, Cipher::aes_128_gcm(), nonce, plaintext, secret)
  }
}

impl Aead for Aes256GcmOpenssl {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    local_decrypt(
      associated_data,
      Cipher::aes_256_gcm(),
      data,
      CryptoError::InvalidAes256GcmData,
      nonce,
      secret,
    )
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    local_encrypt(associated_data, Cipher::aes_256_gcm(), nonce, plaintext, secret)
  }
}

impl Aead for Chacha20Poly1305Openssl {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    local_decrypt(
      associated_data,
      Cipher::chacha20_poly1305(),
      data,
      CryptoError::InvalidChacha20Poly1305Data,
      nonce,
      secret,
    )
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    local_encrypt(associated_data, Cipher::chacha20_poly1305(), nonce, plaintext, secret)
  }
}

fn local_decrypt<'data>(
  associated_data: &[u8],
  cipher: Cipher,
  data: &'data mut [u8],
  error: CryptoError,
  nonce: [u8; AEAD_NONCE_LEN],
  secret: &[u8],
) -> crate::Result<&'data mut [u8]> {
  let (content, tag) = split_content_tag(data, error)?;
  let mut decrypted = decrypt_aead(cipher, secret, Some(&nonce), associated_data, content, &tag)?;
  let decrypted_len = decrypted.len();
  content.get_mut(..decrypted_len).unwrap_or_default().copy_from_slice(&decrypted);
  drop(SensitiveBytes::new_unlocked(&mut decrypted));
  Ok(content)
}

fn local_encrypt(
  associated_data: &[u8],
  cipher: Cipher,
  nonce: [u8; AEAD_NONCE_LEN],
  plaintext: &mut [u8],
  secret: &[u8],
) -> crate::Result<[u8; AEAD_TAG_LEN]> {
  let mut tag = [0u8; AEAD_TAG_LEN];
  let content = encrypt_aead(cipher, secret, Some(&nonce), associated_data, plaintext, &mut tag)?;
  plaintext.get_mut(..content.len()).unwrap_or_default().copy_from_slice(&content);
  Ok(tag)
}
