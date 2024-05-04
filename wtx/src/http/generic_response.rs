use crate::http::Version;

/// HTTP response
pub trait GenericResponse {
  /// Code
  fn code(&self) -> u16;

  /// Version
  fn version(&self) -> Version;
}

impl GenericResponse for () {
  #[inline]
  fn code(&self) -> u16 {
    0
  }

  #[inline]
  fn version(&self) -> Version {
    Version::default()
  }
}
