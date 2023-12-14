use alloc::vec::Vec;

/// Marker trait that has different bounds according to the given set of enabled serializers.
pub trait Serialize<DRSR> {
  /// Tries to encode itself into the specified amount of mutable bytes.
  fn to_bytes(&mut self, bytes: &mut Vec<u8>, drsr: &mut DRSR) -> crate::Result<()>;
}

impl<DRSR> Serialize<DRSR> for () {
  #[inline]
  fn to_bytes(&mut self, _: &mut Vec<u8>, _: &mut DRSR) -> crate::Result<()> {
    Ok(())
  }
}
