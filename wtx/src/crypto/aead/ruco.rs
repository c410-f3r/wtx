use crate::crypto::{
  AEAD_NONCE_LEN, AEAD_TAG_LEN, Aes128GcmRuco, Aes256GcmRuco, Chacha20Poly1305Ruco, CryptoError,
  aead::{Aead, split_content_tag},
};
use aead::{AeadInOut as _, KeyInit};
use aes_gcm::{Aes128Gcm, Aes256Gcm};
use chacha20poly1305::ChaCha20Poly1305;

impl Aead for Aes128GcmRuco {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_parts<'encrypted>(
    associated_data: &[u8],
    data: &'encrypted mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (content, tag) = split_content_tag(data, CryptoError::InvalidAes128GcmData)?;
    <Aes128Gcm as KeyInit>::new(&(*secret).into()).decrypt_inout_detached(
      &(nonce.into()),
      associated_data,
      content.into(),
      &(tag).into(),
    )?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    let tag = <Aes128Gcm as KeyInit>::new(&(*secret).into())
      .encrypt_inout_detached(&(nonce.into()), associated_data, plaintext.into())?
      .into();
    Ok(tag)
  }
}

impl Aead for Aes256GcmRuco {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'encrypted>(
    associated_data: &[u8],
    data: &'encrypted mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (content, tag) = split_content_tag(data, CryptoError::InvalidAes256GcmData)?;
    <Aes256Gcm as KeyInit>::new(&(*secret).into()).decrypt_inout_detached(
      &(nonce.into()),
      associated_data,
      content.into(),
      &(tag).into(),
    )?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    let tag = <Aes256Gcm as KeyInit>::new(&(*secret).into())
      .encrypt_inout_detached(&(nonce.into()), associated_data, plaintext.into())?
      .into();
    Ok(tag)
  }
}

impl Aead for Chacha20Poly1305Ruco {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_parts<'encrypted>(
    associated_data: &[u8],
    data: &'encrypted mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (content, tag) = split_content_tag(data, CryptoError::InvalidChacha20Poly1305Data)?;
    <ChaCha20Poly1305 as KeyInit>::new(secret.into()).decrypt_inout_detached(
      &(nonce.into()),
      associated_data,
      content.into(),
      &(tag.into()),
    )?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    let tag = <ChaCha20Poly1305 as KeyInit>::new(secret.into())
      .encrypt_inout_detached(&(nonce.into()), associated_data, plaintext.into())?
      .into();
    Ok(tag)
  }
}
