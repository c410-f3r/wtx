use crate::{
  http::{GenericHeader, Request, Response, Version},
  misc::_unreachable,
};

impl GenericHeader for httparse::Header<'_> {
  #[inline]
  fn name(&self) -> &[u8] {
    self.name.as_bytes()
  }

  #[inline]
  fn value(&self) -> &[u8] {
    self.value
  }
}

impl Request for httparse::Request<'_, '_> {
  #[inline]
  fn method(&self) -> &[u8] {
    if let Some(el) = self.method {
      el.as_bytes()
    } else {
      _unreachable()
    }
  }

  #[inline]
  fn path(&self) -> &[u8] {
    if let Some(el) = self.path {
      el.as_bytes()
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

impl Response for httparse::Response<'_, '_> {
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
