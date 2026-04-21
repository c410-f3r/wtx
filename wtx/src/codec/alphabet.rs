use core::ops::RangeInclusive;

/// Branchless decoding based on iterative offsets.
//
// FIXME(stable): Copy with the new RangeInclusive
#[derive(Clone, Debug)]
pub(crate) enum DecodeStep {
  /// Maps exactly one byte value to an offset.
  Eq(u8, i16),
  /// Maps an inclusive byte range to an offset.
  Range(RangeInclusive<u8>, i16),
}

/// Branchless encoding based on iterative offsets.
///
/// Fist value is the threshold and the second value is the offset
#[derive(Clone, Copy, Debug)]
pub(crate) enum EncodeStep {
  /// Applies an offset to the running encoded value when the threshold is passed.
  #[expect(unused, reason = "used by other alphabets")]
  Apply(u8, i16),
  /// Applies an offset based on the original value when the threshold is passed.
  Diff(u8, i16),
}

/// Set of allowed characters when encoding or decoding.
pub(crate) trait Alphabet {
  /// First character.
  const BASE: u8;
  /// Decoder passes.
  const DECODER: &'static [DecodeStep];
  /// Encoder passes.
  const ENCODER: &'static [EncodeStep];
  /// Filling data when encoded data is unaligned.
  const PAD: Option<u8>;
}

impl Alphabet for () {
  const BASE: u8 = 0;
  const DECODER: &'static [DecodeStep] = &[];
  const ENCODER: &'static [EncodeStep] = &[];
  const PAD: Option<u8> = None;
}
