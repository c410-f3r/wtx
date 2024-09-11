mod cookie_bytes;
mod cookie_error;
mod cookie_generic;
mod same_site;

use crate::misc::{ArrayVector, Rng, Vector, _shift_copyable_chunks};
pub(crate) use cookie_bytes::CookieBytes;
pub use cookie_error::CookieError;
pub(crate) use cookie_generic::CookieGeneric;
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
pub(crate) fn decrypt(
  buffer: &mut Vector<u8>,
  key: &[u8],
  (name, value): (&[u8], &[u8]),
) -> crate::Result<()> {
  use aes_gcm::{
    aead::{generic_array::GenericArray, AeadInPlace},
    Aes256Gcm, Tag,
  };
  use base64::{engine::general_purpose::STANDARD, Engine};

  let start = buffer.len();
  let (nonce, content, tag) = {
    let expand_len = NONCE_LEN.wrapping_add(value.len()).wrapping_add(TAG_LEN);
    buffer.expand(expand_len, 0)?;
    let actual_len = STANDARD.decode_slice(value, buffer)?;
    buffer.truncate(start.wrapping_add(actual_len));
    #[rustfmt::skip]
    let rslt = if let Some([
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      content @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ]) = buffer.get_mut(start..)
    {
      (
        [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11],
        content,
        [*b0, *b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8, *b9, *b10, *b11, *b12, *b13, *b14, *b15],
      )
    }
    else {
      ([0u8; NONCE_LEN], &mut [][..], [0u8; TAG_LEN])
    };
    rslt
  };
  <Aes256Gcm as aes_gcm::aead::KeyInit>::new(GenericArray::from_slice(key))
    .decrypt_in_place_detached(
      GenericArray::from_slice(&nonce),
      name,
      content,
      Tag::from_slice(&tag),
    )?;
  let idx = start.wrapping_sub(TAG_LEN);
  let _ = _shift_copyable_chunks(0, buffer, [NONCE_LEN..idx]);
  buffer.truncate(idx.wrapping_sub(NONCE_LEN));
  Ok(())
}

#[cfg(feature = "http-cookie-secure")]
#[inline]
pub(crate) fn encrypt<RNG>(
  buffer: &mut Vector<u8>,
  key: &[u8],
  (name, value): (&[u8], &[u8]),
  mut rng: RNG,
) -> crate::Result<()>
where
  RNG: Rng,
{
  use aes_gcm::{
    aead::{generic_array::GenericArray, AeadInPlace},
    Aes256Gcm,
  };
  use base64::{engine::general_purpose::STANDARD, Engine};

  let start = buffer.len();
  let content_len = NONCE_LEN.wrapping_add(value.len()).wrapping_add(TAG_LEN);
  let base64_len = base64::encoded_len(content_len, true).unwrap_or(usize::MAX);
  buffer.expand(base64_len, 0)?;
  buffer.extend_from_slices(&[[0; NONCE_LEN].as_slice(), value, [0; TAG_LEN].as_slice()])?;
  {
    let content_start = start.wrapping_add(base64_len);
    #[rustfmt::skip]
    let Some([
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      content @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ]) = buffer.get_mut(content_start..)
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
    let aes = <Aes256Gcm as aes_gcm::aead::KeyInit>::new(GenericArray::from_slice(key));
    let nonce = [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11];
    let tag = aes.encrypt_in_place_detached(GenericArray::from_slice(&nonce), name, content)?;
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
  let Some((base64, content)) =
    buffer.get_mut(start..).and_then(|el| el.split_at_mut_checked(base64_len))
  else {
    return Ok(());
  };
  let base64_idx = STANDARD.encode_slice(content, base64)?;
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
