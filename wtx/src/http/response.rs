use crate::http::Version;

pub trait Response {
  /// Code
  fn code(&self) -> u16;

  /// Version
  fn version(&self) -> Version;
}

impl Response for () {
  #[inline]
  fn code(&self) -> u16 {
    0
  }

  #[inline]
  fn version(&self) -> Version {
    <_>::default()
  }
}
