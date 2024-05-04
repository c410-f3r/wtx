use crate::http::Version;

/// HTTP request.
pub trait GenericRequest {
  /// Method
  fn method(&self) -> &[u8];

  /// Path
  fn path(&self) -> &[u8];

  /// Version
  fn version(&self) -> Version;
}

impl GenericRequest for () {
  #[inline]
  fn method(&self) -> &[u8] {
    &[]
  }

  #[inline]
  fn path(&self) -> &[u8] {
    &[]
  }

  #[inline]
  fn version(&self) -> Version {
    Version::default()
  }
}
