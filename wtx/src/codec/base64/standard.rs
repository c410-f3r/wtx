use crate::codec::{
  alphabet::{Alphabet, DecodeStep, EncodeStep},
  base64::u8i16,
};

#[derive(Debug)]
pub(crate) struct Standard;

impl Alphabet for Standard {
  const BASE: u8 = b'A';
  const DECODER: &'static [DecodeStep] = DECODER;
  const ENCODER: &'static [EncodeStep] = ENCODER;
  const PAD: Option<u8> = Some(b'=');
}

#[derive(Debug)]
pub(crate) struct StandardNoPad;

impl Alphabet for StandardNoPad {
  const BASE: u8 = b'A';
  const DECODER: &'static [DecodeStep] = DECODER;
  const ENCODER: &'static [EncodeStep] = ENCODER;
  const PAD: Option<u8> = None;
}

const DECODER: &[DecodeStep] = &[
  // 90 - 64 = 26, desired indices are 0..26
  DecodeStep::Range(b'A'..=b'Z', -64),
  // 122 - 70 = 52, desired indices are 26..52
  DecodeStep::Range(b'a'..=b'z', -70),
  // 57 + 5 = 62, desired indices are 52..61
  DecodeStep::Range(b'0'..=b'9', 5),
  DecodeStep::Eq(b'+', 63),
  DecodeStep::Eq(b'/', 64),
];

const ENCODER: &[EncodeStep] = &[
  EncodeStep::Diff(25, 6),
  EncodeStep::Diff(51, -75),
  EncodeStep::Diff(61, -(u8i16(b'+') - 0x1c)),
  EncodeStep::Diff(62, u8i16(b'/') - u8i16(b'+') - 1),
];
