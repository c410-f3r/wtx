use crate::misc::Vector;

/// Marker trait that has different bounds according to the given set of enabled deserializers.
pub trait Deserialize<'de, DRSR>
where
  Self: Sized,
{
  /// Tries to create itself based on the passed amount of bytes.
  fn from_bytes(bytes: &'de [u8], drsr: &mut DRSR) -> crate::Result<Self>;

  /// Similar to [`Self::from_bytes`] but deals with sequences instead of a single element.
  fn seq_from_bytes(
    buffer: &mut Vector<Self>,
    bytes: &'de [u8],
    drsr: &mut DRSR,
  ) -> crate::Result<()>;
}

impl<'de, DRSR> Deserialize<'de, DRSR> for () {
  #[inline]
  fn from_bytes(_: &'de [u8], _: &mut DRSR) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn seq_from_bytes(_: &mut Vector<Self>, _: &'de [u8], _: &mut DRSR) -> crate::Result<()> {
    Ok(())
  }
}
