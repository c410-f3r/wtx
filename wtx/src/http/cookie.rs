mod cookie_error;
#[cfg(feature = "http-session")]
pub(crate) mod cookie_generic;
#[cfg(all(feature = "http-server-framework", feature = "http-session"))]
pub(crate) mod cookie_str;
mod same_site;

#[cfg(feature = "http-session")]
use crate::calendar::CalendarToken;
#[cfg(feature = "http-cookie-secure")]
use crate::{collection::Vector, rng::Rng};
pub use cookie_error::CookieError;
pub use same_site::SameSite;

#[cfg(feature = "http-cookie-secure")]
const NONCE_LEN: usize = 12;
#[cfg(feature = "http-cookie-secure")]
const TAG_LEN: usize = 16;

#[cfg(feature = "http-session")]
static FMT1: &[CalendarToken] = &[
  CalendarToken::AbbreviatedWeekdayName,
  CalendarToken::Comma,
  CalendarToken::Space,
  CalendarToken::TwoDigitDay,
  CalendarToken::Space,
  CalendarToken::AbbreviatedMonthName,
  CalendarToken::Space,
  CalendarToken::FourDigitYear,
  CalendarToken::Space,
  CalendarToken::TwoDigitHour,
  CalendarToken::Colon,
  CalendarToken::TwoDigitMinute,
  CalendarToken::Colon,
  CalendarToken::TwoDigitSecond,
  CalendarToken::Space,
  CalendarToken::Gmt,
];

/// Decrypts the contents of an encrypted cookie that is usually received from a remote peer.
#[cfg(feature = "http-cookie-secure")]
pub fn decrypt_cookie<'buffer>(
  buffer: &'buffer mut Vector<u8>,
  secret: &[u8; 32],
  (name, value): (&[u8], &[u8]),
) -> crate::Result<&'buffer mut [u8]> {
  use crate::collection::ExpansionTy;
  use aes_gcm::{Aes256Gcm, aead::AeadInOut, aes::cipher::Array};
  use base64::{Engine, engine::general_purpose::STANDARD};

  #[rustfmt::skip]
  let (nonce, content, tag) = {
    let start = buffer.len();
    buffer.expand(ExpansionTy::Additional(base64::decoded_len_estimate(value.len())), 0)?;
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
  <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*secret)).decrypt_inout_detached(
    &Array(nonce),
    name,
    content.into(),
    &Array(tag),
  )?;
  Ok(content)
}

/// Encrypts the content of a cookie, which makes it suitable to transmit to remote peers.
#[cfg(feature = "http-cookie-secure")]
pub fn encrypt_cookie<'buffer, RNG>(
  buffer: &'buffer mut Vector<u8>,
  secret: &[u8; 32],
  (name, value): (&[u8], &[u8]),
  mut rng: RNG,
) -> crate::Result<&'buffer mut [u8]>
where
  RNG: Rng,
{
  use crate::collection::ExpansionTy;
  use aes_gcm::{Aes256Gcm, aead::AeadInOut, aes::cipher::Array};
  use base64::{Engine, engine::general_purpose::STANDARD};

  let start = buffer.len();
  let content_len = NONCE_LEN.wrapping_add(value.len()).wrapping_add(TAG_LEN);
  let base64_len = base64::encoded_len(content_len, true).unwrap_or(usize::MAX);
  buffer.expand(ExpansionTy::Additional(base64_len), 0)?;
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
      return Ok(&mut []);
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
    let aes = <Aes256Gcm as aes_gcm::aead::KeyInit>::new(&Array(*secret));
    let nonce = [*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7, *a8, *a9, *a10, *a11];
    let tag = aes.encrypt_inout_detached(&Array(nonce), name, content.into())?;
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
    return Ok(&mut []);
  };
  let base64_idx = STANDARD.encode_slice(&mut *content, base64)?;
  buffer.truncate(start.wrapping_add(base64_idx));
  Ok(buffer.get_mut(start..).unwrap_or_default())
}
