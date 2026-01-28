#[cfg(feature = "base64")]
use crate::{collection::Vector, rng::CryptoRng};

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// Authenticated Encryption with Associated Data
pub trait Aead {
  /// Decrypts data from a single concatenated slice.
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    encrypted_data: &'encrypted mut [u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]>;

  /// Decrypts a base64 encoded string, using the provided buffer for the output.
  #[cfg(feature = "base64")]
  fn decrypt_base64<'buffer>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    encrypted_data: &[u8],
    secret: &[u8; 32],
  ) -> crate::Result<&'buffer mut [u8]>;

  /// Decrypts data from separated slices.
  fn decrypt_vectored(
    associated_data: &[u8],
    content: &mut [u8],
    nonce: [u8; NONCE_LEN],
    secret: &[u8; 32],
    tag: [u8; TAG_LEN],
  ) -> crate::Result<()>;

  /// Encrypts data from a single concatenated slice.
  fn encrypt<'buffer, RNG>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    plaintext_data: &[u8],
    secret: &[u8; 32],
    rng: &mut RNG,
  ) -> crate::Result<&'buffer mut [u8]>
  where
    RNG: CryptoRng;

  /// Encrypts `plaintext_data`, appending the base64 encoded result to the buffer.
  #[cfg(feature = "base64")]
  fn encrypt_base64<'buffer, RNG>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    plaintext_data: &[u8],
    secret: &[u8; 32],
    rng: &mut RNG,
  ) -> crate::Result<&'buffer str>
  where
    RNG: CryptoRng;

  /// Encrypts data outputting separated arrays.
  fn encrypt_vectored<RNG>(
    associated_data: &[u8],
    content: &mut [u8],
    secret: &[u8; 32],
    rng: &mut RNG,
  ) -> crate::Result<([u8; NONCE_LEN], [u8; TAG_LEN])>
  where
    RNG: CryptoRng;
}

impl Aead for () {
  fn decrypt<'encrypted>(
    _: &[u8],
    _: &'encrypted mut [u8],
    _: &[u8; 32],
  ) -> crate::Result<&'encrypted mut [u8]> {
    Ok(&mut [])
  }

  fn decrypt_base64<'buffer>(
    _: &[u8],
    _: &'buffer mut Vector<u8>,
    _: &[u8],
    _: &[u8; 32],
  ) -> crate::Result<&'buffer mut [u8]> {
    Ok(&mut [])
  }

  fn decrypt_vectored(
    _: &[u8],
    _: &mut [u8],
    _: [u8; NONCE_LEN],
    _: &[u8; 32],
    _: [u8; TAG_LEN],
  ) -> crate::Result<()> {
    Ok(())
  }

  fn encrypt<'buffer, RNG>(
    _: &[u8],
    _: &'buffer mut Vector<u8>,
    _: &[u8],
    _: &[u8; 32],
    _: &mut RNG,
  ) -> crate::Result<&'buffer mut [u8]>
  where
    RNG: CryptoRng,
  {
    Ok(&mut [])
  }

  fn encrypt_base64<'buffer, RNG>(
    _: &[u8],
    _: &'buffer mut Vector<u8>,
    _: &[u8],
    _: &[u8; 32],
    _: &mut RNG,
  ) -> crate::Result<&'buffer str>
  where
    RNG: CryptoRng,
  {
    Ok("")
  }

  fn encrypt_vectored<RNG>(
    _: &[u8],
    _: &mut [u8],
    _: &[u8; 32],
    _: &mut RNG,
  ) -> crate::Result<([u8; NONCE_LEN], [u8; TAG_LEN])>
  where
    RNG: CryptoRng,
  {
    Ok(([0; _], [0; _]))
  }
}

#[cfg(feature = "aes-gcm")]
mod aes_gcm {
  #[cfg(feature = "base64")]
  use crate::collection::Vector;
  use crate::{
    crypto::{
      Aes256GcmAesGcm,
      aead::{Aead, NONCE_LEN, TAG_LEN},
    },
    rng::CryptoRng,
  };
  use aes_gcm::{Aes256Gcm, aead::AeadInOut, aes::cipher::Array};

  impl Aead for Aes256GcmAesGcm {
    fn decrypt<'encrypted>(
      associated_data: &[u8],
      encrypted_data: &'encrypted mut [u8],
      secret: &[u8; 32],
    ) -> crate::Result<&'encrypted mut [u8]> {
      #[rustfmt::skip]
      let [
        a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
        content @ ..,
        b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
      ] = encrypted_data
      else {
        return Err(crate::Error::InvalidAes256GcmData);
      };
      <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*secret)).decrypt_inout_detached(
        &Array([*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11]),
        associated_data,
        content.into(),
        &Array([
          *b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8, *b9, *b10, *b11, *b12, *b13, *b14, *b15,
        ]),
      )?;
      Ok(content)
    }

    #[cfg(feature = "base64")]
    fn decrypt_base64<'buffer>(
      associated_data: &[u8],
      buffer: &'buffer mut Vector<u8>,
      encrypted_data: &[u8],
      secret: &[u8; 32],
    ) -> crate::Result<&'buffer mut [u8]> {
      use crate::collection::ExpansionTy;
      use base64::{Engine, engine::general_purpose::STANDARD};

      let start = buffer.len();
      buffer
        .expand(ExpansionTy::Additional(base64::decoded_len_estimate(encrypted_data.len())), 0)?;
      let buffer_slice = buffer.get_mut(start..).unwrap_or_default();
      let len = STANDARD.decode_slice(encrypted_data, buffer_slice)?;
      let end = start.wrapping_add(len);
      Self::decrypt(associated_data, buffer.get_mut(start..end).unwrap_or_default(), secret)
    }

    fn decrypt_vectored(
      associated_data: &[u8],
      content: &mut [u8],
      nonce: [u8; NONCE_LEN],
      secret: &[u8; 32],
      tag: [u8; TAG_LEN],
    ) -> crate::Result<()> {
      <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*secret)).decrypt_inout_detached(
        &Array(nonce),
        associated_data,
        content.into(),
        &Array(tag),
      )?;
      Ok(())
    }

    fn encrypt<'buffer, RNG>(
      associated_data: &[u8],
      buffer: &'buffer mut Vector<u8>,
      plaintext_data: &[u8],
      secret: &[u8; 32],
      rng: &mut RNG,
    ) -> crate::Result<&'buffer mut [u8]>
    where
      RNG: CryptoRng,
    {
      let start = buffer.len();
      let _ = buffer.extend_from_copyable_slices([
        [0; NONCE_LEN].as_slice(),
        plaintext_data,
        [0; TAG_LEN].as_slice(),
      ])?;
      inner_encrypt(associated_data, buffer.get_mut(start..), rng, secret)?;
      Ok(buffer.get_mut(start..).unwrap_or_default())
    }

    // Buffer allocates two areas: one for the resulting base64 and another for intermediary work.
    // FIXME(UPSTREAM): Only one page would be needed if `base64` had support for vectored reads.
    #[cfg(feature = "base64")]
    fn encrypt_base64<'buffer, RNG>(
      associated_data: &[u8],
      buffer: &'buffer mut Vector<u8>,
      plaintext_data: &[u8],
      secret: &[u8; 32],
      rng: &mut RNG,
    ) -> crate::Result<&'buffer str>
    where
      RNG: CryptoRng,
    {
      use crate::{collection::ExpansionTy, misc::SensitiveBytes};
      use base64::{Engine, engine::general_purpose::STANDARD};

      let start = buffer.len();
      let data_len = NONCE_LEN.wrapping_add(plaintext_data.len()).wrapping_add(TAG_LEN);
      let base64_len = base64::encoded_len(data_len, true).unwrap_or(usize::MAX);
      buffer.expand(ExpansionTy::Additional(base64_len), 0)?;
      let _ = buffer.extend_from_copyable_slices([
        [0; NONCE_LEN].as_slice(),
        plaintext_data,
        [0; TAG_LEN].as_slice(),
      ])?;
      inner_encrypt(
        associated_data,
        buffer.get_mut(start.wrapping_add(base64_len)..),
        rng,
        secret,
      )?;
      let slice_mut = buffer.get_mut(start..).and_then(|el| el.split_at_mut_checked(base64_len));
      let Some((base64, content)) = slice_mut else {
        return Ok("");
      };
      let base64_idx = STANDARD.encode_slice(&mut *content, base64)?;
      drop(SensitiveBytes::new_unlocked(content));
      buffer.truncate(start.wrapping_add(base64_idx));
      let bytes = buffer.get_mut(start..).unwrap_or_default();
      // SAFETY: Base64 is ASCII.
      Ok(unsafe { core::str::from_utf8_unchecked(bytes) })
    }

    fn encrypt_vectored<RNG>(
      associated_data: &[u8],
      content: &mut [u8],
      secret: &[u8; 32],
      rng: &mut RNG,
    ) -> crate::Result<([u8; NONCE_LEN], [u8; TAG_LEN])>
    where
      RNG: CryptoRng,
    {
      let mut nonce = [0u8; NONCE_LEN];
      let mut tag = [0u8; TAG_LEN];
      let [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11] = &mut nonce;
      let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15] = &mut tag;
      inner_encrypt_vectored(
        associated_data,
        content,
        [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11],
        rng,
        secret,
        [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15],
      )?;
      Ok((nonce, tag))
    }
  }

  #[inline]
  fn inner_encrypt<RNG>(
    associated_data: &[u8],
    data: Option<&mut [u8]>,
    rng: &mut RNG,
    secret: &[u8; 32],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    #[rustfmt::skip]
    let Some([
        a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
        content @ ..,
        b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ]) = data
    else {
        return Ok(());
    };
    let nonce = [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11];
    let tag = [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15];
    inner_encrypt_vectored(associated_data, content, nonce, rng, secret, tag)
  }

  #[inline]
  fn inner_encrypt_vectored<RNG>(
    associated_data: &[u8],
    content: &mut [u8],
    nonce: [&mut u8; NONCE_LEN],
    rng: &mut RNG,
    secret: &[u8; 32],
    tag: [&mut u8; TAG_LEN],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    let [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11] = nonce;
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15] = tag;
    let [c0, c1, c2, c3, c4, c5, c6, c7, c8, c9, c10, c11, _, _, _, _] = rng.u8_16();
    *a0 = c0;
    *a1 = c1;
    *a2 = c2;
    *a3 = c3;
    *a4 = c4;
    *a5 = c5;
    *a6 = c6;
    *a7 = c7;
    *a8 = c8;
    *a9 = c9;
    *a10 = c10;
    *a11 = c11;
    let aes = <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*secret));
    let nonce = [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11];
    let local_tag = aes.encrypt_inout_detached(&Array(nonce), associated_data, content.into())?;
    let [d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15] = local_tag.into();
    *b0 = d0;
    *b1 = d1;
    *b2 = d2;
    *b3 = d3;
    *b4 = d4;
    *b5 = d5;
    *b6 = d6;
    *b7 = d7;
    *b8 = d8;
    *b9 = d9;
    *b10 = d10;
    *b11 = d11;
    *b12 = d12;
    *b13 = d13;
    *b14 = d14;
    *b15 = d15;
    Ok(())
  }
}
