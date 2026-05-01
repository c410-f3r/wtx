use crate::{
  crypto::{
    Aes128GcmGraviola, Aes256GcmGraviola, Chacha20Poly1305Graviola, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content_tag, write_tag},
  },
  rng::CryptoRng,
};
use graviola::aead::{AesGcm, ChaCha20Poly1305};

impl Aead for Aes128GcmGraviola {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidAes128GcmData)?;
    AesGcm::new(secret).decrypt(&nonce, associated_data, content, &tag)?;
    Ok(content)
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
    local_encrypt(nonce, plaintext, rng, tag, |local_nonce, buffer, out_tag| {
      AesGcm::new(secret).encrypt(local_nonce, associated_data, buffer, out_tag)
    })
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
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidAes256GcmData)?;
    AesGcm::new(secret).decrypt(&nonce, associated_data, content, &tag)?;
    Ok(content)
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
    local_encrypt(nonce, plaintext, rng, tag, |local_nonce, buffer, out_tag| {
      AesGcm::new(secret).encrypt(local_nonce, associated_data, buffer, out_tag)
    })
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
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidChacha20Poly1305Data)?;
    ChaCha20Poly1305::new(*secret).decrypt(&nonce, associated_data, content, &tag)?;
    Ok(content)
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
    local_encrypt(nonce, plaintext, rng, tag, |nonce_bytes, buffer, out_tag| {
      ChaCha20Poly1305::new(*secret).encrypt(nonce_bytes, associated_data, buffer, out_tag)
    })
  }
}

fn local_encrypt<RNG>(
  nonce: [&mut u8; NONCE_LEN],
  plaintext: &mut [u8],
  rng: &mut RNG,
  tag: [&mut u8; TAG_LEN],
  cb: impl FnOnce(&[u8; NONCE_LEN], &mut [u8], &mut [u8; TAG_LEN]),
) -> crate::Result<()>
where
  RNG: CryptoRng,
{
  let local_nonce = generate_nonce(nonce, rng);
  let mut tag_bytes = [0u8; TAG_LEN];
  cb(&local_nonce, plaintext, &mut tag_bytes);
  write_tag(tag_bytes, tag);
  Ok(())
}
