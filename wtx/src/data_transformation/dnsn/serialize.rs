use crate::misc::Vector;

/// Marker trait that has different bounds according to the given set of enabled serializers.
pub trait Serialize<DRSR> {
  /// Tries to encode itself into the specified amount of mutable bytes.
  fn to_bytes(&mut self, bytes: &mut Vector<u8>, drsr: &mut DRSR) -> crate::Result<()>;
}

impl<DRSR> Serialize<DRSR> for () {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vector<u8>, _: &mut DRSR) -> crate::Result<()> {
    Ok(())
  }
}

impl<DRSR, T> Serialize<DRSR> for &mut T
where
  T: Serialize<DRSR>,
{
  #[inline]
  fn to_bytes(&mut self, bytes: &mut Vector<u8>, drsr: &mut DRSR) -> crate::Result<()> {
    (*self).to_bytes(bytes, drsr)
  }
}
