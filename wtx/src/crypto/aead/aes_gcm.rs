use crate::{
  crypto::{
    Aes128GcmRustCrypto, Aes256GcmRustCrypto, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content_tag, write_tag},
  },
  rng::CryptoRng,
};
use aead::{AeadInOut, KeyInit};
use aes_gcm::{Aes128Gcm, Aes256Gcm};

impl Aead for Aes128GcmRustCrypto {
  type Secret = [u8; 16];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidAes128GcmData)?;
    <Aes128Gcm as KeyInit>::new(&(*secret).into()).decrypt_inout_detached(
      &(nonce.into()),
      associated_data,
      content.into(),
      &(tag).into(),
    )?;
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
    let local_tag = <Aes128Gcm as KeyInit>::new(&(*secret).into())
      .encrypt_inout_detached(
        &(generate_nonce(nonce, rng)).into(),
        associated_data,
        plaintext.into(),
      )?
      .into();
    write_tag(local_tag, tag);
    Ok(())
  }
}

impl Aead for Aes256GcmRustCrypto {
  type Secret = [u8; 32];

  #[inline]
  fn decrypt_in_place<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidAes256GcmData)?;
    <Aes256Gcm as KeyInit>::new(&(*secret).into()).decrypt_inout_detached(
      &(nonce.into()),
      associated_data,
      content.into(),
      &(tag).into(),
    )?;
    Ok(content)
  }

  #[inline]
  fn encrypt_parts<RNG>(
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
    let local_tag = <Aes256Gcm as KeyInit>::new(&(*secret).into())
      .encrypt_inout_detached(
        &(generate_nonce(nonce, rng)).into(),
        associated_data,
        plaintext.into(),
      )?
      .into();
    write_tag(local_tag, tag);
    Ok(())
  }
}
