use crate::{
  collection::Clear,
  http::Headers,
  misc::{Lease, LeaseMut, Uri, UriString},
};
use alloc::{boxed::Box, string::String};

static EMPTY_URI_STRING: UriString = UriString::empty(String::new());

/// Groups the elements of an HTTP request/response.
pub trait ReqResData {
  /// See [`Self::body`].
  type Body: ?Sized;

  /// Can be a sequence of bytes, a string, a deserialized element or any other desired type.
  fn body(&self) -> &Self::Body;

  /// See [Headers].
  fn headers(&self) -> &Headers;

  /// See [`UriString`].
  fn uri(&self) -> &UriString;
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

  #[inline]
  fn uri(&self) -> &UriString {
    (*self).uri()
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

  #[inline]
  fn uri(&self) -> &UriString {
    (**self).uri()
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
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
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
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
  }
}

impl ReqResData for () {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &[]
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
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

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
  }
}

impl<B, H, U> ReqResData for (B, H, U)
where
  H: Lease<Headers>,
  U: Lease<UriString>,
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

  #[inline]
  fn uri(&self) -> &UriString {
    self.2.lease()
  }
}

impl<T> ReqResData for Box<T>
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

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
  }
}

impl ReqResData for Headers {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &[]
  }

  #[inline]
  fn headers(&self) -> &Headers {
    self
  }

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
  }
}

impl<S> ReqResData for Uri<S>
where
  S: Lease<str>,
{
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &[]
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> &UriString {
    &EMPTY_URI_STRING
  }
}

/// Mutable version of [`ReqResData`].
pub trait ReqResDataMut: ReqResData {
  /// Can be a sequence of mutable bytes, a mutable string or any other desired type.
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    self.parts_mut().0
  }

  /// Removes all values.
  fn clear(&mut self);

  /// Removes all but URI values.
  fn clear_body_and_headers(&mut self);

  /// Mutable version of [`ReqResData::headers`].
  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    self.parts_mut().1
  }

  /// Mutable parts
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString);
}

impl<T> ReqResDataMut for &mut T
where
  T: ReqResDataMut,
{
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    (**self).body_mut()
  }

  #[inline]
  fn clear(&mut self) {
    (**self).clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    (**self).clear_body_and_headers();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    (**self).headers_mut()
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString) {
    (**self).parts_mut()
  }
}

impl<T> ReqResDataMut for Box<T>
where
  T: ReqResDataMut,
{
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    (**self).body_mut()
  }

  #[inline]
  fn clear(&mut self) {
    (**self).clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    (**self).clear_body_and_headers();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    (**self).headers_mut()
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString) {
    (**self).parts_mut()
  }
}

impl ReqResDataMut for Headers {
  #[inline]
  fn clear(&mut self) {
    self.headers_mut().clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.headers_mut().clear();
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString) {
    (&mut [], self, &EMPTY_URI_STRING)
  }
}

impl<B, H> ReqResDataMut for (B, H)
where
  B: Clear,
  H: LeaseMut<Headers>,
{
  #[inline]
  fn clear(&mut self) {
    self.body_mut().clear();
    self.headers_mut().clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.body_mut().clear();
    self.headers_mut().clear();
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString) {
    (&mut self.0, self.1.lease_mut(), &EMPTY_URI_STRING)
  }
}

impl<B, H, U> ReqResDataMut for (B, H, U)
where
  B: Clear,
  H: LeaseMut<Headers>,
  U: LeaseMut<UriString>,
{
  #[inline]
  fn clear(&mut self) {
    self.0.clear();
    self.1.lease_mut().clear();
    self.2.lease_mut().clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.0.clear();
    self.1.lease_mut().clear();
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, &UriString) {
    (&mut self.0, self.1.lease_mut(), self.2.lease())
  }
}
