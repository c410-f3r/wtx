pub(crate) mod cookie_bytes;
mod cookie_error;
pub(crate) mod cookie_generic;
mod same_site;

use crate::misc::{ArrayVector, Rng, Vector};
pub use cookie_error::CookieError;
use core::str;
pub use same_site::SameSite;

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

static FMT1: &str = "%a, %d %b %Y %H:%M:%S GMT";
static FMT2: &str = "%A, %d-%b-%y %H:%M:%S GMT";
static FMT3: &str = "%a %b %e %H:%M:%S %Y";
static FMT4: &str = "%a, %d-%b-%Y %H:%M:%S GMT";

#[cfg(feature = "http-cookie-secure")]
#[inline]
pub(crate) fn decrypt<'buffer>(
  buffer: &'buffer mut Vector<u8>,
  key: &[u8; 32],
  (name, value): (&[u8], &[u8]),
) -> crate::Result<&'buffer mut [u8]> {
  use crate::misc::BufferMode;
  use aes_gcm::{Aes256Gcm, aead::AeadInPlace, aes::cipher::Array};
  use base64::{Engine, engine::general_purpose::STANDARD};

  #[rustfmt::skip]
  let (nonce, content, tag) = {
    let start = buffer.len();
    buffer.expand(BufferMode::Additional(base64::decoded_len_estimate(value.len())), 0)?;
    let buffer_slice = buffer.get_mut(start..).unwrap_or_default();
    let len = STANDARD.decode_slice(value, buffer_slice)?;
    let end = start.wrapping_add(len);
    if let Some([
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      content @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ]) = buffer.get_mut(start..end)
    {
      (
        [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11],
        content,
        [*b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8, *b9, *b10, *b11, *b12, *b13, *b14, *b15],
      )
    }
    else {
      ([0u8; NONCE_LEN], &mut [][..], [0u8; TAG_LEN])
    }
  };
  <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*key)).decrypt_in_place_detached(
    &Array(nonce),
    name,
    content,
    &Array(tag),
  )?;
  Ok(content)
}

#[cfg(feature = "http-cookie-secure")]
#[inline]
pub(crate) fn encrypt<RNG>(
  buffer: &mut Vector<u8>,
  key: &[u8; 32],
  (name, value): (&[u8], &[u8]),
  mut rng: RNG,
) -> crate::Result<()>
where
  RNG: Rng,
{
  use crate::misc::BufferMode;
  use aes_gcm::{Aes256Gcm, aead::AeadInPlace, aes::cipher::Array};
  use base64::{Engine, engine::general_purpose::STANDARD};

  let start = buffer.len();
  let content_len = NONCE_LEN.wrapping_add(value.len()).wrapping_add(TAG_LEN);
  let base64_len = base64::encoded_len(content_len, true).unwrap_or(usize::MAX);
  buffer.expand(BufferMode::Additional(base64_len), 0)?;
  let _ = buffer.extend_from_copyable_slices([
    [0; NONCE_LEN].as_slice(),
    value,
    [0; TAG_LEN].as_slice(),
  ])?;
  {
    #[rustfmt::skip]
    let Some([
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      content @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ]) = buffer.get_mut(start.wrapping_add(base64_len)..)
    else {
      return Ok(());
    };
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
    let aes = <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*key));
    let nonce = [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11];
    let tag = aes.encrypt_in_place_detached(&Array(nonce), name, content)?;
    let [d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15] = tag.into();
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
  };
  let slice_mut = buffer.get_mut(start..).and_then(|el| el.split_at_mut_checked(base64_len));
  let Some((base64, content)) = slice_mut else {
    return Ok(());
  };
  let base64_idx = STANDARD.encode_slice(&mut *content, base64)?;
  buffer.truncate(start.wrapping_add(base64_idx));
  Ok(())
}

#[inline]
fn make_lowercase<const UPPER_BOUND: usize>(buffer: &mut ArrayVector<u8, 12>, slice: &[u8]) {
  buffer.clear();
  let sub_slice = slice.get(..slice.len().min(UPPER_BOUND)).unwrap_or_default();
  let _rslt = buffer.extend_from_copyable_slice(sub_slice);
  buffer.make_ascii_lowercase();
}
