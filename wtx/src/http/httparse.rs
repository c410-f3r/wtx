#![allow(
  // All methods are internally called only after parsing
  clippy::unreachable
)]

use crate::http::{Header, Http1Header, Request, Response, Version};

impl<'buffer> Header for httparse::Header<'buffer> {
  #[inline]
  fn name(&self) -> &[u8] {
    self.name.as_bytes()
  }

  #[inline]
  fn value(&self) -> &[u8] {
    self.value
  }
}

impl<'buffer> Http1Header for httparse::Header<'buffer> {}

impl<'buffer, 'headers> Request for httparse::Request<'headers, 'buffer> {
  #[inline]
  fn method(&self) -> &[u8] {
    if let Some(el) = self.method {
      el.as_bytes()
    } else {
      unreachable!()
    }
  }

  #[inline]
  fn path(&self) -> &[u8] {
    if let Some(el) = self.path {
      el.as_bytes()
    } else {
      unreachable!()
    }
  }

  #[inline]
  fn version(&self) -> Version {
    match self.version {
      Some(0) => Version::Http1,
      Some(1) => Version::Http2,
      _ => {
        unreachable!()
      }
    }
  }
}

impl<'buffer, 'headers> Response for httparse::Response<'headers, 'buffer> {
  #[inline]
  fn code(&self) -> u16 {
    if let Some(el) = self.code {
      el
    } else {
      unreachable!()
    }
  }

  #[inline]
  fn version(&self) -> Version {
    match self.version {
      Some(0) => Version::Http1,
      Some(1) => Version::Http2,
      _ => {
        unreachable!()
      }
    }
  }
}
