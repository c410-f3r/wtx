use graviola::aead::AesGcm;

use crate::{
  crypto::{
    Aes128GcmGraviola, Aes256GcmGraviola, Chacha20Poly1305Graviola, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content, write_tag},
  },
  rng::CryptoRng,
};

impl Aead for Aes128GcmGraviola {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let error = CryptoError::InvalidAes128GcmData;
    let (nonce, content) = split_nonce_content(encrypted_data, error)?;
    let ciphertext_len = content.len().checked_sub(TAG_LEN).ok_or(error)?;
    let (ciphertext, tag_bytes) = content.split_at_mut(ciphertext_len);
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(tag_bytes);
    AesGcm::new(secret).decrypt(&nonce, associated_data, ciphertext, &tag).map_err(|_| error)?;
    Ok(ciphertext)
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
    let nonce_bytes = generate_nonce(nonce, rng);
    let mut tag_bytes = [0u8; TAG_LEN];
    AesGcm::new(secret).encrypt(&nonce_bytes, associated_data, plaintext, &mut tag_bytes);
    write_tag(tag_bytes, tag);
    Ok(())
  }
}

impl Aead for Aes256GcmGraviola {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let error = CryptoError::InvalidAes256GcmData;
    let (nonce, content) = split_nonce_content(encrypted_data, error)?;
    let ciphertext_len = content.len().checked_sub(TAG_LEN).ok_or(error)?;
    let (ciphertext, tag_bytes) = content.split_at_mut(ciphertext_len);
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(tag_bytes);
    AesGcm::new(secret).decrypt(&nonce, associated_data, ciphertext, &tag).map_err(|_| error)?;
    Ok(ciphertext)
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
    let nonce_bytes = generate_nonce(nonce, rng);
    let mut tag_bytes = [0u8; TAG_LEN];
    AesGcm::new(secret).encrypt(&nonce_bytes, associated_data, plaintext, &mut tag_bytes);
    write_tag(tag_bytes, tag);
    Ok(())
  }
}

impl Aead for Chacha20Poly1305Graviola {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let error = CryptoError::InvalidChacha20Poly1305Data;
    let (nonce, content) = split_nonce_content(encrypted_data, error)?;
    let ciphertext_len = content.len().checked_sub(TAG_LEN).ok_or(error)?;
    let (ciphertext, tag_bytes) = content.split_at_mut(ciphertext_len);
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(tag_bytes);
    graviola::aead::ChaCha20Poly1305::new(*secret)
      .decrypt(&nonce, associated_data, ciphertext, &tag)
      .map_err(|_| error)?;
    Ok(ciphertext)
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
    let nonce_bytes = generate_nonce(nonce, rng);
    let mut tag_bytes = [0u8; TAG_LEN];
    graviola::aead::ChaCha20Poly1305::new(*secret).encrypt(
      &nonce_bytes,
      associated_data,
      plaintext,
      &mut tag_bytes,
    );
    write_tag(tag_bytes, tag);
    Ok(())
  }
}
