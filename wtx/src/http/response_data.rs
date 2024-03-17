use core::borrow::Borrow;

use crate::http::Headers;

/// Groups the body and the headers of a response into a single structure.
pub trait ResponseData {
  /// See [Self::body]
  type Body<'body>
  where
    Self: 'body;

  /// Can be a sequence of bytes, a string, a deserialized element or any other desired type.
  fn body(&self) -> Self::Body<'_>;

  /// See [Headers].
  fn headers(&self) -> &Headers;
}

impl<T> ResponseData for &T
where
  T: ResponseData,
{
  type Body<'body> = T::Body<'body>
  where
    Self: 'body;

  #[inline]
  fn body(&self) -> Self::Body<'_> {
    (*self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (*self).headers()
  }
}

impl<T> ResponseData for &mut T
where
  T: ResponseData,
{
  type Body<'body> = T::Body<'body>
  where
    Self: 'body;

  #[inline]
  fn body(&self) -> Self::Body<'_> {
    (**self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (**self).headers()
  }
}

impl<B, H> ResponseData for (B, H)
where
  H: Borrow<Headers>,
{
  type Body<'body> = &'body B
  where
    Self: 'body;

  #[inline]
  fn body(&self) -> Self::Body<'_> {
    &self.0
  }

  #[inline]
  fn headers(&self) -> &Headers {
    self.1.borrow()
  }
}

#[cfg(feature = "http2")]
impl ResponseData for crate::http2::ReqResBuffer {
  type Body<'body> = &'body [u8];

  #[inline]
  fn body(&self) -> Self::Body<'_> {
    &self.data
  }

  #[inline]
  fn headers(&self) -> &Headers {
    &self.headers
  }
}
