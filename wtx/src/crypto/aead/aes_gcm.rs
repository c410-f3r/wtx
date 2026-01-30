
use crate::{
  crypto::{
    Aes128GcmRustCrypto, Aes256GcmRustCrypto, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content_tag, write_tag},
  },
  rng::CryptoRng,
};
use aes_gcm::{
  Aes128Gcm, Aes256Gcm,
  aead::{AeadInOut, KeyInit},
  aes::cipher::Array,
};

impl Aead<16> for Aes128GcmRustCrypto {
  #[inline]
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 16],
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidAes128GcmData)?;
    <Aes128Gcm as KeyInit>::new(&Array(*secret)).decrypt_inout_detached(
      &Array(nonce),
      associated_data,
      content.into(),
      &Array(tag),
    )?;
    Ok(content)
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
    let local_tag = <Aes128Gcm as KeyInit>::new(&Array(*secret))
      .encrypt_inout_detached(
        &Array(generate_nonce(nonce, rng)),
        associated_data,
        plaintext.into(),
      )?
      .into();
    write_tag(local_tag, tag);
    Ok(())
  }
}

impl Aead<32> for Aes256GcmRustCrypto {
  #[inline]
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidAes256GcmData)?;
    <Aes256Gcm as KeyInit>::new(&Array(*secret)).decrypt_inout_detached(
      &Array(nonce),
      associated_data,
      content.into(),
      &Array(tag),
    )?;
    Ok(content)
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
    let local_tag = <Aes256Gcm as KeyInit>::new(&Array(*secret))
      .encrypt_inout_detached(
        &Array(generate_nonce(nonce, rng)),
        associated_data,
        plaintext.into(),
      )?
      .into();
    write_tag(local_tag, tag);
    Ok(())
  }
}
