use crate::misc::from_utf8_basic;
use alloc::{string::String, vec::Vec};

/// This trait only exists because of the lack of `impl TryFrom<&[u8]> for String` but such
/// implementation probably will never be a thing.
pub trait FromBytes {
  /// Creates itself from a sequence of bytes.
  fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
  where
    Self: Sized;
}

impl FromBytes for String {
  #[inline]
  fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
  where
    Self: Sized,
  {
    Ok(from_utf8_basic(bytes)?.into())
  }
}

impl FromBytes for Vec<u8> {
  #[inline]
  fn from_bytes(bytes: &[u8]) -> crate::Result<Self>
  where
    Self: Sized,
  {
    Ok(bytes.into())
  }
}
