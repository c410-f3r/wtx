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

#[cfg(feature = "httparse")]
mod httparse {
  use crate::{
    http::{GenericRequest, Version},
    misc::hints::_unlikely_unreachable,
  };

  impl GenericRequest for httparse::Request<'_, '_> {
    #[inline]
    fn method(&self) -> &[u8] {
      if let Some(el) = self.method { el.as_bytes() } else { _unlikely_unreachable() }
    }

    #[inline]
    fn path(&self) -> &[u8] {
      if let Some(el) = self.path { el.as_bytes() } else { _unlikely_unreachable() }
    }

    #[inline]
    fn version(&self) -> Version {
      match self.version {
        Some(0) => Version::Http1,
        Some(1) => Version::Http1_1,
        _ => _unlikely_unreachable(),
      }
    }
  }
}
