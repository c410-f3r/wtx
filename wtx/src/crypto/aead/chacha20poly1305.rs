
use crate::{
  crypto::{
    Chacha20Poly1305RustCrypto, CryptoError,
    aead::{Aead, NONCE_LEN, TAG_LEN, generate_nonce, split_nonce_content_tag, write_tag},
  },
  rng::CryptoRng,
};
use chacha20poly1305::{ChaCha20Poly1305, aead::KeyInit};

impl Aead<32> for Chacha20Poly1305RustCrypto {
  #[inline]
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]> {
    let (nonce, content, tag) =
      split_nonce_content_tag(encrypted_data, CryptoError::InvalidChacha20Poly1305Data)?;
    <ChaCha20Poly1305 as KeyInit>::new(secret.into()).decrypt_inout_detached(
      nonce.into(),
      associated_data,
      content.into(),
      tag.into(),
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
    let local_tag = <ChaCha20Poly1305 as KeyInit>::new(secret.into())
      .encrypt_inout_detached(generate_nonce(nonce, rng).into(), associated_data, plaintext.into())?
      .into();
    write_tag(local_tag, tag);
    Ok(())
  }
}
