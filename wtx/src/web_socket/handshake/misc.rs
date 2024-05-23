use crate::rng::Rng;
use base64::{engine::general_purpose::STANDARD, Engine};
use sha1::{Digest, Sha1};

pub(crate) fn derived_key<'buffer>(buffer: &'buffer mut [u8; 30], key: &[u8]) -> &'buffer [u8] {
  let mut sha1 = Sha1::new();
  sha1.update(key);
  sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
  base64_from_array(&sha1.finalize().into(), buffer)
}

pub(crate) fn gen_key<'buffer>(buffer: &'buffer mut [u8; 26], rng: &mut impl Rng) -> &'buffer [u8] {
  base64_from_array(&rng.u8_16(), buffer)
}

fn base64_from_array<'output, const I: usize, const O: usize>(
  input: &[u8; I],
  output: &'output mut [u8; O],
) -> &'output [u8] {
  const {
    let rslt = if let Some(elem) = base64::encoded_len(I, false) { elem } else { 0 };
    assert!(O >= rslt);
  }
  let len = STANDARD.encode_slice(input, output).unwrap_or_default();
  output.get(..len).unwrap_or_default()
}
