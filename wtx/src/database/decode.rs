use crate::database::Database;

/// Similar to `TryFrom`. Avoids problems with coherence and has an additional `E` type.
pub trait Decode<'de, D>: Sized
where
  D: Database,
{
  /// Performs the conversion.
  fn decode(dv: &D::DecodeValue<'de>) -> Result<Self, D::Error>;
}

impl Decode<'_, ()> for &str {
  #[inline]
  fn decode(_: &()) -> Result<Self, crate::Error> {
    Ok("")
  }
}

impl Decode<'_, ()> for u32 {
  #[inline]
  fn decode(_: &()) -> Result<Self, crate::Error> {
    Ok(0)
  }
}

impl Decode<'_, ()> for u64 {
  #[inline]
  fn decode(_: &()) -> Result<Self, crate::Error> {
    Ok(0)
  }
}
