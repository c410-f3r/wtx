#![allow(
  // All methods are called after parsing
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
