/// Similar to `TryFrom`. Avoids problems with coherence and has an additional `E` type.
pub trait Decode<C, E, I>: Sized
where
  E: From<crate::Error>,
{
  /// Performs the conversion.
  fn decode(input: I) -> Result<Self, E>;
}

impl<I, T> Decode<(), crate::Error, I> for T
where
  T: Default,
{
  #[inline]
  fn decode(_: I) -> Result<Self, crate::Error> {
    Ok(T::default())
  }
}
