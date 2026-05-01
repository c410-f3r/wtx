use crate::{
  crypto::{
    Aes128GcmOpenssl, Aes256GcmOpenssl, Chacha20Poly1305Openssl, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content_tag, write_tag},
  },
  misc::SensitiveBytes,
  rng::CryptoRng,
};
use openssl::symm::{Cipher, decrypt_aead, encrypt_aead};

impl Aead for Aes128GcmOpenssl {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    local_decrypt(
      associated_data,
      Cipher::aes_128_gcm(),
      encrypted_data,
      CryptoError::InvalidAes128GcmData,
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
    local_encrypt(associated_data, Cipher::aes_128_gcm(), nonce, plaintext, rng, secret, tag)
  }
}

impl Aead for Aes256GcmOpenssl {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    local_decrypt(
      associated_data,
      Cipher::aes_256_gcm(),
      encrypted_data,
      CryptoError::InvalidAes256GcmData,
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
    local_encrypt(associated_data, Cipher::aes_256_gcm(), nonce, plaintext, rng, secret, tag)
  }
}

impl Aead for Chacha20Poly1305Openssl {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    local_decrypt(
      associated_data,
      Cipher::chacha20_poly1305(),
      encrypted_data,
      CryptoError::InvalidChacha20Poly1305Data,
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
    local_encrypt(associated_data, Cipher::chacha20_poly1305(), nonce, plaintext, rng, secret, tag)
  }
}

fn local_decrypt<'encrypted>(
  associated_data: &[u8],
  cipher: Cipher,
  encrypted_data: &'encrypted mut [u8],
  error: CryptoError,
  secret: &[u8],
) -> crate::Result<&'encrypted mut [u8]> {
  let (nonce, content, tag) = split_nonce_content_tag(encrypted_data, error)?;
  let mut decrypted = decrypt_aead(cipher, secret, Some(&nonce), associated_data, content, &tag)?;
  let decrypted_len = decrypted.len();
  content.get_mut(..decrypted_len).unwrap_or_default().copy_from_slice(&decrypted);
  drop(SensitiveBytes::new_unlocked(&mut decrypted));
  Ok(content)
}

fn local_encrypt<RNG>(
  associated_data: &[u8],
  cipher: Cipher,
  nonce: [&mut u8; NONCE_LEN],
  plaintext: &mut [u8],
  rng: &mut RNG,
  secret: &[u8],
  tag_bytes_out: [&mut u8; TAG_LEN],
) -> crate::Result<()>
where
  RNG: CryptoRng,
{
  let nonce_bytes = generate_nonce(nonce, rng);
  let mut tag = [0u8; TAG_LEN];
  let content =
    encrypt_aead(cipher, secret, Some(&nonce_bytes), associated_data, plaintext, &mut tag)?;
  plaintext.get_mut(..content.len()).unwrap_or_default().copy_from_slice(&content);
  write_tag(tag, tag_bytes_out);
  Ok(())
}
