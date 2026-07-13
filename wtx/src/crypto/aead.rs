#[cfg(feature = "crypto-aws-lc-rs")]
mod aws_lc_rs;
pub(crate) mod global;
#[cfg(feature = "crypto-graviola")]
mod graviola;
#[cfg(feature = "crypto-ring")]
mod ring;

use crate::{
  codec::{
    Base64Alphabet, base64_decode, base64_decoded_len_ub, base64_encode, base64_encoded_len,
  },
  collections::{ExpansionTy, Vector},
  crypto::{AEAD_NONCE_LEN, AEAD_TAG_LEN, CryptoError, dummy_crypto_call},
  misc::SensitiveBytes,
};
use core::marker::PhantomData;

/// Authenticated Encryption with Associated Data
pub trait Aead {
  /// Secret
  type Secret;

  /// Decrypts data in-place with the associated affixes
  fn decrypt_parts<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; AEAD_NONCE_LEN],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]>;

  /// Encrypts plaintext content with the associated nonce.
  fn encrypt_parts(
    associated_data: &[u8],
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]>;

  // ***** PROVIDED *****

  /// Decrypts a base64 encoded string, using the provided buffer for the output.
  #[inline]
  fn decrypt_base64_to_buffer<'buffer>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    data: &[u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'buffer mut [u8]> {
    let additional = base64_decoded_len_ub(data.len());
    let begin = buffer.len();
    buffer.expand(ExpansionTy::Additional(additional), 0)?;
    let buffer_slice = buffer.get_mut(begin..).unwrap_or_default();
    let len = base64_decode(Base64Alphabet::UrlNoPad, data, buffer_slice)?.len();
    buffer.truncate(begin.wrapping_add(len));
    Self::decrypt_in_place(associated_data, buffer.get_mut(begin..).unwrap_or_default(), secret)
  }

  /// Decrypts data in-place.
  ///
  /// `data` should contain any associated affix.
  #[inline]
  fn decrypt_in_place<'data>(
    associated_data: &[u8],
    data: &'data mut [u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    let Some((nonce, payload)) = data.split_first_chunk_mut() else {
      return Err(CryptoError::InvalidAesData.into());
    };
    Self::decrypt_parts(associated_data, payload, *nonce, secret)
  }

  /// Encrypts data into a dedicated buffer
  #[inline]
  fn encrypt_to_buffer<'buffer>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &[u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'buffer mut [u8]> {
    let begin = buffer.len();
    let _ = buffer.extend_from_copyable_slices([
      nonce.as_slice(),
      plaintext,
      [0; AEAD_TAG_LEN].as_slice(),
    ])?;
    let plaintext_begin = begin.wrapping_add(AEAD_NONCE_LEN);
    let plaintext_end = buffer.len().wrapping_sub(AEAD_TAG_LEN);
    let tag = Self::encrypt_parts(
      associated_data,
      nonce,
      buffer.get_mut(plaintext_begin..plaintext_end).unwrap_or_default(),
      secret,
    )?;
    if let Some(elem) = buffer.last_chunk_mut::<AEAD_TAG_LEN>() {
      elem.copy_from_slice(&tag);
    }
    Ok(buffer.get_mut(begin..).unwrap_or_default())
  }

  /// Encrypts `plaintext`, appending the base64 encoded result into `buffer`.
  //
  // Buffer allocates two areas: one for the resulting base64 and another for intermediary work.
  #[inline]
  fn encrypt_to_buffer_base64<'buffer>(
    associated_data: &[u8],
    buffer: &'buffer mut Vector<u8>,
    nonce: [u8; AEAD_NONCE_LEN],
    plaintext: &[u8],
    secret: &Self::Secret,
  ) -> crate::Result<&'buffer str> {
    let begin = buffer.len();
    let data_len = AEAD_NONCE_LEN.wrapping_add(plaintext.len()).wrapping_add(AEAD_TAG_LEN);
    let base64_len = base64_encoded_len(data_len, true).unwrap_or(usize::MAX);
    buffer.expand(ExpansionTy::Additional(base64_len), 0)?;
    let _ = buffer.extend_from_copyable_slices([
      nonce.as_slice(),
      plaintext,
      [0; AEAD_TAG_LEN].as_slice(),
    ])?;
    let plaintext_begin = begin.wrapping_add(base64_len).wrapping_add(AEAD_NONCE_LEN);
    let plaintext_end = buffer.len().wrapping_sub(AEAD_TAG_LEN);
    let tag = Self::encrypt_parts(
      associated_data,
      nonce,
      buffer.get_mut(plaintext_begin..plaintext_end).unwrap_or_default(),
      secret,
    )?;
    if let Some(elem) = buffer.last_chunk_mut::<AEAD_TAG_LEN>() {
      elem.copy_from_slice(&tag);
    }
    let slice_mut = buffer.get_mut(begin..).and_then(|el| el.split_at_mut_checked(base64_len));
    let Some((base64, content)) = slice_mut else {
      return Ok("");
    };
    let base64_idx = base64_encode(Base64Alphabet::UrlNoPad, content, base64)?.len();
    drop(SensitiveBytes::new_unlocked(content));
    buffer.truncate(begin.wrapping_add(base64_idx));
    let bytes = buffer.get_mut(begin..).unwrap_or_default();
    // SAFETY: Base64 is ASCII.
    Ok(unsafe { core::str::from_utf8_unchecked(bytes) })
  }
}

/// Dummy [`Aead`] implementation used when no backend is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AeadDummy<S>(PhantomData<S>);

impl<S> Aead for AeadDummy<S> {
  type Secret = S;

  #[inline]
  fn decrypt_parts<'data>(
    _: &[u8],
    _: &'data mut [u8],
    _: [u8; AEAD_NONCE_LEN],
    _: &Self::Secret,
  ) -> crate::Result<&'data mut [u8]> {
    dummy_crypto_call();
  }

  #[inline]
  fn encrypt_parts(
    _: &[u8],
    _: [u8; AEAD_NONCE_LEN],
    _: &mut [u8],
    _: &Self::Secret,
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    dummy_crypto_call();
  }
}

#[cfg(feature = "crypto-graviola")]
fn split_content_tag(
  data: &mut [u8],
  error: CryptoError,
) -> crate::Result<(&mut [u8], [u8; AEAD_TAG_LEN])> {
  let Some((content, tag)) = data.split_last_chunk_mut() else {
    return Err(error.into());
  };
  Ok((content, *tag))
}
