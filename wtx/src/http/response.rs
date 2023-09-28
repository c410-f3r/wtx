use crate::http::Version;

pub trait Response {
  /// Version
  fn version(&self) -> Version;
}

impl Response for () {
  #[inline]
  fn version(&self) -> Version {
    <_>::default()
  }
}
