use crate::{http::Headers, misc::Lease};

/// Groups the body and the headers of a response into a single structure.
pub trait ResponseData {
  /// See [Self::body]
  type Body: ?Sized;

  /// Can be a sequence of bytes, a string, a deserialized element or any other desired type.
  fn body(&self) -> &Self::Body;

  /// See [Headers].
  fn headers(&self) -> &Headers;
}

impl<T> ResponseData for &T
where
  T: ResponseData,
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

impl<T> ResponseData for &mut T
where
  T: ResponseData,
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

impl<B, H> ResponseData for (B, H)
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
impl ResponseData for crate::http2::ReqResBuffer {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.data
  }

  #[inline]
  fn headers(&self) -> &Headers {
    &self.headers
  }
}
