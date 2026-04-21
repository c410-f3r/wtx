use crate::codec::{
  alphabet::{Alphabet, DecodeStep, EncodeStep},
  base64::u8i16,
};

#[derive(Debug)]
pub(crate) struct Url;

impl Alphabet for Url {
  const BASE: u8 = b'A';
  const DECODER: &'static [DecodeStep] = DECODER;
  const ENCODER: &'static [EncodeStep] = ENCODER;
  const PAD: Option<u8> = Some(b'=');
}

#[derive(Debug)]
pub(crate) struct UrlNoPad;

impl Alphabet for UrlNoPad {
  const BASE: u8 = b'A';
  const DECODER: &'static [DecodeStep] = DECODER;
  const ENCODER: &'static [EncodeStep] = ENCODER;
  const PAD: Option<u8> = None;
}

const DECODER: &[DecodeStep] = &[
  DecodeStep::Range(b'A'..=b'Z', -64),
  DecodeStep::Range(b'a'..=b'z', -70),
  DecodeStep::Range(b'0'..=b'9', 5),
  DecodeStep::Eq(b'-', 63),
  DecodeStep::Eq(b'_', 64),
];

const ENCODER: &[EncodeStep] = &[
  EncodeStep::Diff(25, 6),
  EncodeStep::Diff(51, -75),
  EncodeStep::Diff(61, -(u8i16(b'-') - 0x20)),
  EncodeStep::Diff(62, u8i16(b'_') - u8i16(b'-') - 1),
];
