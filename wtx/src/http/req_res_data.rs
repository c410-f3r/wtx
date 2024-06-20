use crate::{http::Headers, misc::Lease};

/// Groups the body and the headers of an HTTP request/response.
pub trait ReqResData {
  /// See [`Self::body`].
  type Body: ?Sized;

  /// Can be a sequence of bytes, a string, a deserialized element or any other desired type.
  fn body(&self) -> &Self::Body;

  /// See [Headers].
  fn headers(&self) -> &Headers;
}

impl<T> ReqResData for &T
where
  T: ReqResData,
{
  type Body = T::Body;

  #[inline]
  fn body(&self) -> &Self::Body {
    (*self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (*self).headers()
  }
}

impl<T> ReqResData for &mut T
where
  T: ReqResData,
{
  type Body = T::Body;

  #[inline]
  fn body(&self) -> &Self::Body {
    (**self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (**self).headers()
  }
}

impl ReqResData for &[u8] {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    self
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new(0) }
  }
}

impl<const N: usize> ReqResData for [u8; N] {
  type Body = [u8; N];

  #[inline]
  fn body(&self) -> &Self::Body {
    self
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new(0) }
  }
}

impl<B, H> ReqResData for (B, H)
where
  H: Lease<Headers>,
{
  type Body = B;

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.0
  }

  #[inline]
  fn headers(&self) -> &Headers {
    self.1.lease()
  }
}

#[cfg(feature = "http2")]
impl ReqResData for crate::http2::ReqResBuffer {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.body
  }

  #[inline]
  fn headers(&self) -> &Headers {
    &self.headers
  }
}
