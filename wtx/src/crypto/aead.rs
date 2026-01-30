#[cfg(feature = "aes-gcm")]
mod aes_gcm;
#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs;
#[cfg(feature = "chacha20poly1305")]
mod chacha20poly1305;

use crate::{collection::Vector, crypto::CryptoError, rng::CryptoRng};

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// Authenticated Encryption with Associated Data
pub trait Aead<const S: usize> {
  /// Decrypts data in-place.
  ///
  /// `data` should contain any associated affix.
  fn decrypt<'encrypted>(
    associated_data: &[u8],
    data: &'encrypted mut [u8],
    secret: &[u8; S],
  ) -> crate::Result<&'encrypted mut [u8]>;

  /// Decrypts a base64 encoded string, using the provided buffer for the output.
  #[cfg(feature = "base64")]
  fn decrypt_base64<'buffer>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    encrypted_data: &[u8],
    secret: &[u8; S],
  ) -> crate::Result<&'buffer mut [u8]> {
    use crate::collection::ExpansionTy;
    use base64::{Engine, engine::general_purpose::STANDARD};

    let additional = base64::decoded_len_estimate(encrypted_data.len());
    let begin = buffer.len();
    buffer.expand(ExpansionTy::Additional(additional), 0)?;
    let buffer_slice = buffer.get_mut(begin..).unwrap_or_default();
    let len = STANDARD.decode_slice(encrypted_data, buffer_slice)?;
    buffer.truncate(begin.wrapping_add(len));
    Self::decrypt(associated_data, buffer.get_mut(begin..).unwrap_or_default(), secret)
  }

  /// Encrypts `plaintext`, appending the base64 encoded result into `buffer`.
  //
  // Buffer allocates two areas: one for the resulting base64 and another for intermediary work.
  // FIXME(UPSTREAM): Only one page would be needed if `base64` had support for vectored reads.
  #[cfg(feature = "base64")]
  fn encrypt_base64<'buffer, RNG>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    plaintext: &[u8],
    rng: &mut RNG,
    secret: &[u8; S],
  ) -> crate::Result<&'buffer str>
  where
    RNG: CryptoRng,
  {
    use crate::{collection::ExpansionTy, misc::SensitiveBytes};
    use base64::{Engine, engine::general_purpose::STANDARD};

    let begin = buffer.len();
    let data_len = NONCE_LEN.wrapping_add(plaintext.len()).wrapping_add(TAG_LEN);
    let base64_len = base64::encoded_len(data_len, true).unwrap_or(usize::MAX);
    buffer.expand(ExpansionTy::Additional(base64_len), 0)?;
    let _ = buffer.extend_from_copyable_slices([
      [0; NONCE_LEN].as_slice(),
      plaintext,
      [0; TAG_LEN].as_slice(),
    ])?;
    Self::encrypt_concatenated_data(
      associated_data,
      buffer.get_mut(begin.wrapping_add(base64_len)..).unwrap_or_default(),
      rng,
      secret,
    )?;
    let slice_mut = buffer.get_mut(begin..).and_then(|el| el.split_at_mut_checked(base64_len));
    let Some((base64, content)) = slice_mut else {
      return Ok("");
    };
    let base64_idx = STANDARD.encode_slice(&mut *content, base64)?;
    drop(SensitiveBytes::new_unlocked(content));
    buffer.truncate(begin.wrapping_add(base64_idx));
    let bytes = buffer.get_mut(begin..).unwrap_or_default();
    // SAFETY: Base64 is ASCII.
    Ok(unsafe { core::str::from_utf8_unchecked(bytes) })
  }

  /// Encrypts data that contains the plaintext content as well as the associated affixes.
  fn encrypt_concatenated_data<RNG>(
    associated_data: &[u8],
    data: &mut [u8],
    rng: &mut RNG,
    secret: &[u8; S],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    #[rustfmt::skip]
    let [
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      plaintext @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ] = data
    else {
      return Err(CryptoError::InvalidAesData.into());
    };
    let nonce = [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11];
    let tag = [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15];
    Self::encrypt_vectored_data(associated_data, nonce, plaintext, rng, secret, tag)
  }

  /// Encrypts data into a dedicated buffer
  fn encrypt_into_buffer<'buffer, RNG>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    plaintext: &[u8],
    rng: &mut RNG,
    secret: &[u8; S],
  ) -> crate::Result<&'buffer mut [u8]>
  where
    RNG: CryptoRng,
  {
    let start = buffer.len();
    let _ = buffer.extend_from_copyable_slices([
      [0; NONCE_LEN].as_slice(),
      plaintext,
      [0; TAG_LEN].as_slice(),
    ])?;
    Self::encrypt_concatenated_data(
      associated_data,
      buffer.get_mut(start..).unwrap_or_default(),
      rng,
      secret,
    )?;
    Ok(buffer.get_mut(start..).unwrap_or_default())
  }

  /// Encrypts plaintext content with the associated affixes.
  ///
  /// This is an internal low-level operation. You should probably call other encryption methods.
  fn encrypt_vectored_data<RNG>(
    associated_data: &[u8],
    nonce: [&mut u8; NONCE_LEN],
    plaintext: &mut [u8],
    rng: &mut RNG,
    secret: &[u8; S],
    tag: [&mut u8; TAG_LEN],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng;
}

impl<const S: usize> Aead<S> for () {
  fn decrypt<'encrypted>(
    _: &[u8],
    _: &'encrypted mut [u8],
    _: &[u8; S],
  ) -> crate::Result<&'encrypted mut [u8]> {
    Ok(&mut [])
  }

  fn encrypt_vectored_data<RNG>(
    _: &[u8],
    _: [&mut u8; NONCE_LEN],
    _: &mut [u8],
    _: &mut RNG,
    _: &[u8; S],
    _: [&mut u8; TAG_LEN],
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    Ok(())
  }
}

#[cfg(any(feature = "aes-gcm", feature = "aws-lc-rs", feature = "chacha20poly1305"))]
fn generate_nonce<RNG: CryptoRng>(nonce: [&mut u8; NONCE_LEN], rng: &mut RNG) -> [u8; NONCE_LEN] {
  let [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11] = nonce;
  let [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, _, _, _, _] = rng.u8_16();
  *a0 = b0;
  *a1 = b1;
  *a2 = b2;
  *a3 = b3;
  *a4 = b4;
  *a5 = b5;
  *a6 = b6;
  *a7 = b7;
  *a8 = b8;
  *a9 = b9;
  *a10 = b10;
  *a11 = b11;
  [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11]
}

#[cfg(feature = "aws-lc-rs")]
fn split_nonce_content(
  data: &mut [u8],
  error: CryptoError,
) -> crate::Result<([u8; NONCE_LEN], &mut [u8])> {
  let [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, content @ ..] = data else {
    return Err(error.into());
  };
  let nonce = [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11];
  Ok((nonce, content))
}

#[cfg(any(feature = "aes-gcm", feature = "aws-lc-rs", feature = "chacha20poly1305"))]
fn split_nonce_content_tag(
  data: &mut [u8],
  error: CryptoError,
) -> crate::Result<([u8; NONCE_LEN], &mut [u8], [u8; TAG_LEN])> {
  #[rustfmt::skip]
  let [
    a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
    content @ ..,
    b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
  ] = data
  else {
    return Err(error.into());
  };
  let nonce = [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11];
  let tag = [*b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8, *b9, *b10, *b11, *b12, *b13, *b14, *b15];
  Ok((nonce, content, tag))
}

#[cfg(any(feature = "aes-gcm", feature = "aws-lc-rs", feature = "chacha20poly1305"))]
fn write_tag(from: [u8; TAG_LEN], to: [&mut u8; TAG_LEN]) {
  for (dest, src) in to.into_iter().zip(from) {
    *dest = src;
  }
}
