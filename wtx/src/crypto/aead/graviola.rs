use crate::crypto::{
  Aes128GcmGraviola, Aes256GcmGraviola, Chacha20Poly1305Graviola, CryptoError,
  aead::{AEAD_NONCE_LEN, AEAD_TAG_LEN, Aead, split_content_tag},
};
use graviola::aead::{AesGcm, ChaCha20Poly1305};

impl Aead for Aes128GcmGraviola {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    let (content, tag) = split_content_tag(data, CryptoError::InvalidAes128GcmData)?;
    AesGcm::new(secret).decrypt(&nonce, associated_data, content, &tag)?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    let mut tag = [0; AEAD_TAG_LEN];
    AesGcm::new(secret).encrypt(&nonce, associated_data, plaintext, &mut tag);
    Ok(tag)
  }
}

impl Aead for Aes256GcmGraviola {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    let (content, tag) = split_content_tag(data, CryptoError::InvalidAes256GcmData)?;
    AesGcm::new(secret).decrypt(&nonce, associated_data, content, &tag)?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    let mut tag = [0; AEAD_TAG_LEN];
    AesGcm::new(secret).encrypt(&nonce, associated_data, plaintext, &mut tag);
    Ok(tag)
  }
}

impl Aead for Chacha20Poly1305Graviola {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    let (content, tag) = split_content_tag(data, CryptoError::InvalidChacha20Poly1305Data)?;
    ChaCha20Poly1305::new(*secret).decrypt(&nonce, associated_data, content, &tag)?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    let mut tag = [0; AEAD_TAG_LEN];
    ChaCha20Poly1305::new(*secret).encrypt(&nonce, associated_data, plaintext, &mut tag);
    Ok(tag)
  }
}
