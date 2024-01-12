use crate::database::Database;

/// Similar to `TryFrom`. Avoids problems with coherence and has an additional `E` type.
pub trait Decode<'de, D>: Sized
where
  D: Database,
{
  /// Performs the conversion.
  fn decode(input: &D::DecodeValue<'de>) -> Result<Self, D::Error>;
}

impl<'de> Decode<'de, ()> for &str {
  #[inline]
  fn decode(_: &()) -> Result<Self, crate::Error> {
    Ok("")
  }
}

impl<'de> Decode<'de, ()> for u32 {
  #[inline]
  fn decode(_: &()) -> Result<Self, crate::Error> {
    Ok(0)
  }
}

impl<'de> Decode<'de, ()> for u64 {
  #[inline]
  fn decode(_: &()) -> Result<Self, crate::Error> {
    Ok(0)
  }
}
