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

#[cfg(feature = "httparse")]
mod httparse {
  use crate::{
    http::{GenericResponse, Version},
    misc::_unreachable,
  };

  impl GenericResponse for httparse::Response<'_, '_> {
    #[inline]
    fn code(&self) -> u16 {
      if let Some(el) = self.code {
        el
      } else {
        _unreachable()
      }
    }

    #[inline]
    fn version(&self) -> Version {
      match self.version {
        Some(0) => Version::Http1,
        Some(1) => Version::Http1_1,
        _ => _unreachable(),
      }
    }
  }
}
