use core::fmt::Display;

/// Marker trait that has different bounds according to the given set of enabled deserializers.
pub trait Deserialize<DRSR>
where
  Self: Sized,
{
  /// Tries to create itself based on the passed amount of bytes.
  fn from_bytes(bytes: &[u8], drsr: &mut DRSR) -> crate::Result<Self>;

  /// Similar to [`Self::from_bytes`] but deals with sequences instead of a single element.
  fn seq_from_bytes<E>(
    bytes: &[u8],
    drsr: &mut DRSR,
    cb: impl FnMut(Self) -> Result<(), E>,
  ) -> Result<(), E>
  where
    E: Display + From<crate::Error>;
}

impl<DRSR> Deserialize<DRSR> for () {
  #[inline]
  fn from_bytes(_: &[u8], _: &mut DRSR) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn seq_from_bytes<E>(
    _: &[u8],
    _: &mut DRSR,
    _: impl FnMut(Self) -> Result<(), E>,
  ) -> Result<(), E>
  where
    E: From<crate::Error>,
  {
    Ok(())
  }
}
