/// If a codec should encode or decode a slice or push new elements
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CodecMode<'fill, B = ()> {
  /// New elements will be pushed into the buffer
  Extend(B),
  /// New elements will be moved into the slice.
  Fill(&'fill mut [u8]),
}
