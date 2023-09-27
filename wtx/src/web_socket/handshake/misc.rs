use crate::{misc::from_utf8_opt, rng::Rng};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha1::{Digest, Sha1};

pub(crate) fn derived_key<'buffer>(buffer: &'buffer mut [u8; 30], key: &[u8]) -> &'buffer str {
  let mut sha1 = Sha1::new();
  sha1.update(key);
  sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
  base64_from_array(&sha1.finalize().into(), buffer)
}

pub(crate) fn gen_key<'buffer>(buffer: &'buffer mut [u8; 26], rng: &mut impl Rng) -> &'buffer str {
  base64_from_array(&rng.u8_16(), buffer)
}

#[allow(
    // False positive
    clippy::arithmetic_side_effects,
    // Buffer has enough capacity and `base64` already returns a valid string
    clippy::unwrap_used
)]
fn base64_from_array<'output, const I: usize, const O: usize>(
  input: &[u8; I],
  output: &'output mut [u8; O],
) -> &'output str {
  fn div_ceil(x: usize, y: usize) -> usize {
    let fun = || {
      let num = x.checked_add(y)?.checked_sub(1)?;
      num.checked_div(y)
    };
    fun().unwrap_or_default()
  }
  assert!(O >= div_ceil(I, 3).wrapping_mul(4));
  let len = STANDARD.encode_slice(input, output).unwrap();
  from_utf8_opt(output.get(..len).unwrap_or_default()).unwrap()
}
