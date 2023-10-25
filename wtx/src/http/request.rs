use crate::http::Version;

pub trait Request {
  /// Method
  fn method(&self) -> &[u8];

  /// Path
  fn path(&self) -> &[u8];

  /// Version
  fn version(&self) -> Version;
}

impl Request for () {
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
    <_>::default()
  }
}
